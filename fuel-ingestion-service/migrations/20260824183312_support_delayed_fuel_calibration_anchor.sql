-- ==========================================================
-- Support Delayed Fuel Calibration Anchoring
-- ==========================================================
--
-- Guided fuel calibration may begin before the installer knows
-- the absolute quantity of fuel already present in the tank.
--
-- During that period ORBI records:
--
-- - physical KUM distance;
-- - cumulative fuel added or removed;
-- - unresolved absolute litres.
--
-- Once an absolute anchor becomes available, such as:
--
-- - confirmed EMPTY;
-- - confirmed FULL;
-- - independently measured fuel quantity;
--
-- the session and its previously captured points can be resolved
-- into absolute litres.
--
-- This migration updates the original guided-calibration schema
-- so active and paused sessions can remain unresolved while
-- completed sessions require resolved absolute quantities.
-- ==========================================================


-- ==========================================================
-- Fuel Calibration Sessions
-- ==========================================================

-- Starting and ending litres are no longer always known when a
-- session begins.
ALTER TABLE fuel_calibration_sessions
    ALTER COLUMN starting_litres DROP NOT NULL;

ALTER TABLE fuel_calibration_sessions
    ALTER COLUMN ending_litres DROP NOT NULL;


-- The original schema assumed calibration always progressed from
-- lower litres to higher litres.
--
-- That is no longer valid because ORBI also supports draining
-- calibration:
--
-- 60 L
-- ↓
-- 40 L
-- ↓
-- 20 L
-- ↓
-- 0 L
--
-- Session direction is now represented through cumulative fuel
-- change on the captured calibration points.
ALTER TABLE fuel_calibration_sessions
    DROP CONSTRAINT IF EXISTS fuel_calibration_session_litre_order;


-- ----------------------------------------------------------
-- Absolute Calibration Anchor
-- ----------------------------------------------------------
--
-- The anchor describes a moment during the guided session where
-- the absolute amount of fuel becomes known.
--
-- Example:
--
-- cumulative_change_litres = +60
-- anchor_absolute_litres   = 200
--
-- therefore:
--
-- session starting litres = 200 - 60 = 140
--
-- The same mechanism also supports draining:
--
-- cumulative_change_litres = -60
-- anchor_absolute_litres   = 0
--
-- therefore:
--
-- session starting litres = 0 - (-60) = 60
-- ----------------------------------------------------------

ALTER TABLE fuel_calibration_sessions
    ADD COLUMN anchor_cumulative_change_litres DOUBLE PRECISION;

ALTER TABLE fuel_calibration_sessions
    ADD COLUMN anchor_absolute_litres DOUBLE PRECISION;

ALTER TABLE fuel_calibration_sessions
    ADD COLUMN anchor_established_at TIMESTAMPTZ;


-- Anchor cumulative change may be positive, zero, or negative.
--
-- It must only be finite.
ALTER TABLE fuel_calibration_sessions
    ADD CONSTRAINT fuel_calibration_session_anchor_change_finite
    CHECK (
        anchor_cumulative_change_litres IS NULL
        OR (
            anchor_cumulative_change_litres <> 'Infinity'::float8
            AND anchor_cumulative_change_litres <> '-Infinity'::float8
            AND anchor_cumulative_change_litres <> 'NaN'::float8
        )
    );


-- Absolute litres may never be negative.
--
-- The upper tank-capacity rule is enforced in the Rust domain/service
-- layer because tank capacity belongs to fuel_calibration_profiles,
-- not directly to this table.
ALTER TABLE fuel_calibration_sessions
    ADD CONSTRAINT fuel_calibration_session_anchor_absolute_non_negative
    CHECK (
        anchor_absolute_litres IS NULL
        OR (
            anchor_absolute_litres >= 0
            AND anchor_absolute_litres <> 'Infinity'::float8
            AND anchor_absolute_litres <> '-Infinity'::float8
            AND anchor_absolute_litres <> 'NaN'::float8
        )
    );


-- The three anchor fields represent one logical object.
--
-- Either:
--
-- no anchor exists
--
-- or:
--
-- cumulative change
-- absolute litres
-- established timestamp
--
-- must all exist together.
ALTER TABLE fuel_calibration_sessions
    ADD CONSTRAINT fuel_calibration_session_anchor_completeness
    CHECK (
        (
            anchor_cumulative_change_litres IS NULL
            AND anchor_absolute_litres IS NULL
            AND anchor_established_at IS NULL
        )
        OR
        (
            anchor_cumulative_change_litres IS NOT NULL
            AND anchor_absolute_litres IS NOT NULL
            AND anchor_established_at IS NOT NULL
        )
    );


-- A completed guided session must have absolute quantities resolved.
--
-- Active and paused sessions are deliberately allowed to leave
-- starting_litres and ending_litres NULL.
ALTER TABLE fuel_calibration_sessions
    ADD CONSTRAINT fuel_calibration_session_resolved_when_completed
    CHECK (
        status <> 'completed'
        OR (
            starting_litres IS NOT NULL
            AND ending_litres IS NOT NULL
        )
    );


-- ==========================================================
-- Fuel Calibration Session Points
-- ==========================================================

-- The original "litres" column represented an absolute resolved
-- quantity.
--
-- Rename it so the meaning is explicit.
ALTER TABLE fuel_calibration_session_points
    RENAME COLUMN litres TO resolved_litres;


-- Absolute litres are not necessarily known when the physical
-- observation is first captured.
ALTER TABLE fuel_calibration_session_points
    ALTER COLUMN resolved_litres DROP NOT NULL;


-- ----------------------------------------------------------
-- Relative Fuel Change
-- ----------------------------------------------------------
--
-- Every observation records how much fuel has been added or
-- removed relative to the beginning of the guided session.
--
-- Examples:
--
--  0 L   = initial observation
-- +20 L  = twenty litres added
-- +40 L  = forty litres added
-- -20 L  = twenty litres removed
-- -40 L  = forty litres removed
--
-- Before an absolute anchor exists:
--
-- resolved_litres = NULL
--
-- After anchoring:
--
-- resolved_litres = session_start_litres
--                   + cumulative_change_litres
-- ----------------------------------------------------------

ALTER TABLE fuel_calibration_session_points
    ADD COLUMN cumulative_change_litres DOUBLE PRECISION NOT NULL
        DEFAULT 0;


ALTER TABLE fuel_calibration_session_points
    ADD CONSTRAINT fuel_calibration_point_change_finite
    CHECK (
        cumulative_change_litres <> 'Infinity'::float8
        AND cumulative_change_litres <> '-Infinity'::float8
        AND cumulative_change_litres <> 'NaN'::float8
    );


-- Resolved absolute litres remain optional until an anchor exists.
--
-- When present they must be valid non-negative finite quantities.
ALTER TABLE fuel_calibration_session_points
    ADD CONSTRAINT fuel_calibration_point_resolved_non_negative
    CHECK (
        resolved_litres IS NULL
        OR (
            resolved_litres >= 0
            AND resolved_litres <> 'Infinity'::float8
            AND resolved_litres <> '-Infinity'::float8
            AND resolved_litres <> 'NaN'::float8
        )
    );


-- ==========================================================
-- Point Uniqueness
-- ==========================================================

-- The old unique index used absolute litres:
--
-- session_id + litres
--
-- That no longer works because absolute litres may be NULL while
-- calibration is still unanchored.
DROP INDEX IF EXISTS unique_fuel_calibration_session_litres;


-- Within one guided session, each cumulative-change position should
-- normally represent one captured verification point.
--
-- Example:
--
-- session start      → 0 L change
-- first addition     → +20 L
-- second addition    → +40 L
--
-- Capturing the same cumulative quantity twice would otherwise
-- create ambiguous calibration evidence.
CREATE UNIQUE INDEX unique_fuel_calibration_session_change
    ON fuel_calibration_session_points (
        session_id,
        cumulative_change_litres
    );
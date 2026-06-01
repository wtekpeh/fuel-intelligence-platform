import { useMapReplayStore } from "../../store/mapReplayStore";

function ReplayControls() {
  const isReplayMode = useMapReplayStore((state) => state.isReplayMode);

  const isPlaying = useMapReplayStore((state) => state.isPlaying);

  const playbackSpeedMs = useMapReplayStore((state) => state.playbackSpeedMs);

  const startReplay = useMapReplayStore((state) => state.startReplay);

  const stopReplay = useMapReplayStore((state) => state.stopReplay);

  const play = useMapReplayStore((state) => state.play);

  const pause = useMapReplayStore((state) => state.pause);

  const setPlaybackSpeedMs = useMapReplayStore(
    (state) => state.setPlaybackSpeedMs,
  );

  return (
    <div className="replay-controls">
      {!isReplayMode ? (
        <button type="button" onClick={startReplay}>
          Start Replay
        </button>
      ) : (
        <>
          <button type="button" onClick={isPlaying ? pause : play}>
            {isPlaying ? "Pause" : "Play"}
          </button>

          <button type="button" onClick={stopReplay}>
            Stop
          </button>

          <select
            value={playbackSpeedMs}
            onChange={(event) => setPlaybackSpeedMs(Number(event.target.value))}
          >
            <option value={2000}>0.5x</option>
            <option value={1000}>1x</option>
            <option value={500}>2x</option>
            <option value={250}>4x</option>
          </select>
        </>
      )}
    </div>
  );
}

export default ReplayControls;

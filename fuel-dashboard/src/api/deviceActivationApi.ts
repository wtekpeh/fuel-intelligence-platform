import { httpClient } from "./httpClient";

/**
 * A device assigned to a customer organization
 * and currently eligible for activation.
 */
export interface ActivationEligibleDevice {
  inventory_device_id: string;
  device_code: string;
  organization_id: string;
  organization_name: string;
}

/**
 * Request to activate an assigned device
 * against an existing customer asset.
 */
export interface ActivateDeviceRequest {
  inventory_device_id: string;
  asset_id: string;
}

/**
 * Retrieve devices that the authenticated customer
 * is authorized to activate.
 */
export async function listActivationEligibleDevices(): Promise<
  ActivationEligibleDevice[]
> {
  const response = await httpClient.get<ActivationEligibleDevice[]>(
    "/api/devices/activation-eligible",
  );

  return response.data;
}

/**
 * Activate a customer-assigned device.
 *
 * The backend returns the UUID of the newly
 * provisioned operational device.
 */
export async function activateCustomerDevice(
  request: ActivateDeviceRequest,
): Promise<string> {
  const response = await httpClient.post<string>(
    "/api/devices/activate",
    request,
  );

  return response.data;
}

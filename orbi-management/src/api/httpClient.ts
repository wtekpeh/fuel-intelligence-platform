import axios from "axios";

import { keycloak } from "../auth/keycloak";

export const httpClient = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL ?? "http://127.0.0.1:8080",

  headers: {
    "Content-Type": "application/json",
  },
});

httpClient.interceptors.request.use(async (config) => {
  await keycloak.updateToken(30);

  if (!keycloak.token) {
    throw new Error("No Keycloak access token available.");
  }

  config.headers.Authorization = `Bearer ${keycloak.token}`;

  return config;
});

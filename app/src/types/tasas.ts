import type { RegisterRateFormValues } from "./schemas";

export type RegistrarTasaRequest = RegisterRateFormValues;

export type TasaDTO = RegistrarTasaRequest & {
  id: string;
  eliminada: boolean;
};

export type RegistrarTasaRequest = {
  moneda_origen: string;
  moneda_destino: string;
  valor: string;
  fecha: string;
  fuente: string;
};

export type TasaDTO = RegistrarTasaRequest & {
  id: string;
  eliminada: boolean;
};

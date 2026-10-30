export type ConsultarPatrimonioRequest = {
  fecha_consulta: string;
};

export type SaldoCuenta = {
  cuenta_id: string;
  nombre_cuenta: string;
  moneda: string;
  saldo: string;
};

export type SubtotalMoneda = {
  moneda: string;
  subtotal: string;
};

export type TasaUsada = {
  moneda_origen: string;
  moneda_destino: string;
  valor: string;
  fecha: string;
  fuente: string;
  es_inversa: boolean;
};

export type TasaFaltante = {
  moneda_origen: string;
  moneda_destino: string;
  fecha_limite: string;
};

export type Patrimonio = {
  moneda_base: string;
  saldos_por_cuenta: SaldoCuenta[];
  subtotales_por_moneda: SubtotalMoneda[];
  total_consolidado: string | null;
  tasas_usadas: TasaUsada[];
  tasas_faltantes: TasaFaltante[];
};

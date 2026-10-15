export interface ErrorValidacionDTO {
  campo: "nombre" | "moneda_codigo" | "saldo_inicial" | "tipo";
  mensaje: string;
}
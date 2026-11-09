"use client";

import Decimal from "decimal.js";
import { GlobalForm, FormField } from "@/components/forms";
import { correctMovementSchema, type CorrectMovementFormValues } from "@/types/schemas";
import { applyBackendValidationError } from "@/types/validation";
import { clearAmountInput, minorUnits } from "@/types/account-money";
import type { Currency } from "@/types/accounts";
import type { MovimientoDTO, CorregirMovimientoRequest } from "@/types/movements";
import type { UseFormReturn } from "react-hook-form";

function amount(value: Decimal, currency: Currency) {
  return `${currency.simbolo} ${value.toString()}`;
}

type Props = {
  movement: MovimientoDTO;
  currency: Currency;
  onCorrect: (request: CorregirMovimientoRequest) => Promise<MovimientoDTO>;
  onRegistered: (movement: MovimientoDTO) => void | Promise<void>;
  onClose: () => void;
};

export function CorrectionMovementForm({
  movement,
  currency,
  onCorrect,
  onRegistered,
  onClose,
}: Props) {
  const submit = async (
    values: CorrectMovementFormValues,
    form: UseFormReturn<CorrectMovementFormValues>,
  ) => {
    try {
      const corrected = await onCorrect(values);
      await onRegistered(corrected);
      onClose();
    } catch (error: unknown) {
      const failure = error as { campo?: unknown; mensaje?: unknown };
      if (failure.campo === "saldo") {
        form.setError("root.server", {
          type: "business",
          message: typeof failure.mensaje === "string" ? failure.mensaje : "La corrección dejaría el saldo negativo.",
        });
        form.setValue("confirmar_saldo_negativo", true);
        return;
      }
      if (!applyBackendValidationError(error, form.setError)) {
        form.setError("root.server", {
          type: "server",
          message: typeof failure.mensaje === "string" ? failure.mensaje : "No se pudo corregir el movimiento.",
        });
      }
    }
  };

  return (
    <section className="correction-dialog" aria-labelledby="correction-dialog-title">
      <div className="movement-dialog-heading">
        <div>
          <h2 id="correction-dialog-title">Corregir movimiento</h2>
          <p>Monto registrado: {amount(minorUnits(movement.monto), currency)}</p>
        </div>
      </div>
      <GlobalForm<CorrectMovementFormValues>
        schema={correctMovementSchema}
        defaultValues={{
          movimiento_id: movement.id,
          monto_correcto: clearAmountInput(movement.monto),
          confirmar_saldo_negativo: false,
        }}
        onSubmit={submit}
      >
        {(form) => (
          <>
            <FormField<CorrectMovementFormValues>
              name="monto_correcto"
              label={`Monto correcto (${currency.codigo})`}
              help={`Hasta ${currency.decimales} ${currency.decimales === 1 ? "decimal" : "decimales"}.`}
            >
              {(props) => (
                <input
                  {...props}
                  type="text"
                  inputMode="decimal"
                  autoFocus
                  disabled={form.formState.isSubmitting}
                  onInput={(event) => {
                    event.currentTarget.value = clearAmountInput(event.currentTarget.value);
                    void form.trigger("monto_correcto");
                  }}
                  {...form.register("monto_correcto")}
                />
              )}
            </FormField>
            <div className="correction-actions">
              <button type="button" className="account-secondary" onClick={onClose} disabled={form.formState.isSubmitting}>
                Cancelar
              </button>
              <button className="account-primary" type="submit" disabled={form.formState.isSubmitting}>
                {form.formState.isSubmitting
                  ? "Corrigiendo..."
                  : form.getValues("confirmar_saldo_negativo")
                    ? "Confirmar y guardar"
                    : "Guardar corrección"}
              </button>
            </div>
          </>
        )}
      </GlobalForm>
    </section>
  );
}

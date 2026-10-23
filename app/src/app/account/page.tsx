"use client";

import { Suspense, useState } from "react";
import { useSearchParams } from "next/navigation";
import { useAccounts } from "@/features/accounts/useAccounts";
import { useAccountMovements } from "@/features/movements/useAccountMovements";
import { useMovementCategories } from "@/features/movements/useMovementCategories";
import { AccountDetail } from "@/components/accounts/account-detail";
import { registerMovementFromForm } from "@/features/movements/register-movement";
import { correctMovementFromForm } from "@/features/movements/correct-movement";
import { createCategory } from "@/services/createCategory";
import "../accounts.css";

function AccountDetailContent() {
  const searchParams = useSearchParams();
  const { accounts, currencies, refresh: refreshAccounts } = useAccounts();
  const accountId = searchParams.get("id");
  const { movements, error: movementsError, refresh: refreshMovements } =
    useAccountMovements(accountId);
  const {
    categories,
    loading: categoriesLoading,
    error: categoriesError,
    refresh: refreshCategories,
  } = useMovementCategories();
  const [formOpen, setFormOpen] = useState(false);

  const account = accounts.find((item) => item.id === accountId);
  if (!account) {
    return <main className="account-detail-loading">Cargando cuenta...</main>;
  }

  return (
    <AccountDetail
      key={account.id}
      account={account}
      currency={currencies.find((item) => item.codigo === account.moneda_codigo)}
      movements={movements}
      movementsError={movementsError}
      formOpen={formOpen}
      onOpenMovementForm={() => setFormOpen(true)}
      onCloseMovementForm={() => setFormOpen(false)}
      onMovementRegistered={async () => {
        await Promise.all([refreshMovements(), refreshAccounts()]);
        setFormOpen(false);
      }}
      onRegister={registerMovementFromForm}
      onCorrect={correctMovementFromForm}
      categories={categories}
      categoriesLoading={categoriesLoading}
      categoriesError={categoriesError}
      onRetryCategories={() => void refreshCategories()}
      onCreateCategory={async (request) => {
        const category = await createCategory(request);
        await refreshCategories();
        return category;
      }}
    />
  );
}

export default function AccountDetailPage() {
  return (
    <Suspense fallback={<main className="account-detail-loading">Cargando cuenta...</main>}>
      <AccountDetailContent />
    </Suspense>
  );
}

"use client";
 
 import { useEffect, useState } from "react";
 import type { Account, CreateAccountInput, Currency } from "@/types/accounts";
 import { createAccount } from "@/services/createAcount";
 import { listCurrencies } from "@/services/listCurrencies";
 import { listAccounts } from "@/services/listAccounts";
 
 export function useAccounts() {
   const [accounts, setAccounts] = useState<Account[]>([]);
   const [currencies, setCurrencies] = useState<Currency[]>([]);
   const [announcement, setAnnouncement] = useState("");
 

    // C
    useEffect(() => {
     async function loadCurrencies() {
       try {
         setCurrencies(await listCurrencies());
       } catch (error: unknown) {
         console.error("No se pudo cargar el catálogo de monedas.", error);
         setAnnouncement("No se pudo cargar el catálogo de monedas.");
       }
     }
 
     void loadCurrencies();
    }, []);

    // Carga las cuentas guardadas en la BD
    useEffect(() => {
      async function loadAccounts() {
        try {
          setAccounts(await listAccounts());
        } catch (error: unknown) {
          console.error("No se pudieron cargar las cuentas:", error);
          setAnnouncement("No se pudieron cargar las cuentas");
        }
      }

      void loadAccounts()
    }, [])

    async function create(input: CreateAccountInput) {
      const account = await createAccount(input);
 
      setAccounts((current) => [...current, account]);
      setAnnouncement(`Cuenta ${account.nombre} creada correctamente.`);
 
      return account;
    }

    async function refresh() {
      const nextAccounts = await listAccounts();
      setAccounts(nextAccounts);
      return nextAccounts;
    }
 
    return {
      accounts,
      currencies,
      announcement,
      create,
      refresh,
    };
  }

"use client";

import Image from "next/image";
import Link from "next/link";
import { useState } from "react";
import logo from "@/img/logo-rm-bg.png";

export function AccountsSidebar({
  active = "accounts",
}: {
  active?: "accounts" | "connection";
}) {
  const [open, setOpen] = useState(false);

  return (
    <>
      <button
        type="button"
        className="sidebar-toggle"
        aria-expanded={open}
        aria-controls="accounts-navigation"
        onClick={() => setOpen((isOpen) => !isOpen)}
      >
        <span aria-hidden="true">☰</span>
        <span className="sr-only">Abrir navegación</span>
      </button>
      {open && (
        <button
          type="button"
          className="sidebar-backdrop"
          aria-label="Cerrar navegación"
          onClick={() => setOpen(false)}
        />
      )}
      <aside
        id="accounts-navigation"
        className={`accounts-sidebar${open ? " is-open" : ""}`}
      >
        <Link href="/" className="accounts-brand" onClick={() => setOpen(false)}>
          <Image
            className="brand-logo"
            src={logo}
            alt=""
            width={40}
            height={40}
            priority
          />
          <span>
            Cuentas Claras<small>finanzas locales</small>
          </span>
        </Link>
        <p className="eyebrow">Principal</p>
        <nav aria-label="Principal">
          <Link
            href="/"
            aria-current={active === "accounts" ? "page" : undefined}
            className={active === "accounts" ? "nav-current" : undefined}
            onClick={() => setOpen(false)}
          >
            <span aria-hidden="true">▦</span> Mis cuentas
          </Link>
          <button disabled>
            <span aria-hidden="true">◎</span> Grupos <small>Pronto</small>
          </button>
        </nav>
        <div className="sidebar-divider" />
        <p className="eyebrow">Sistema</p>
        <nav aria-label="Sistema">
          <Link
            href="/conexion/"
            aria-current={active === "connection" ? "page" : undefined}
            className={active === "connection" ? "nav-current" : undefined}
            onClick={() => setOpen(false)}
          >
            <span aria-hidden="true">⌁</span> Conexión
          </Link>
        </nav>
      </aside>
    </>
  );
}

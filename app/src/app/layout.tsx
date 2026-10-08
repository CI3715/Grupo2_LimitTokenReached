import type {
  Metadata,
} from "next";

import type {
  ReactNode,
} from "react";

import "./globals.css";

export const metadata: Metadata = {
  title:
    "Cuentas Claras",

  description:
    "Sistema de conexión de Cuentas Claras",
};

type RootLayoutProps = {
  children: ReactNode;
};

export default function RootLayout({
  children,
}: RootLayoutProps) {
  return (
    <html
      lang="es"
      suppressHydrationWarning
    >
      <body>
        {children}
      </body>
    </html>
  );
}
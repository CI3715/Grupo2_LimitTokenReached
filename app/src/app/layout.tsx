import type { Metadata } from "next";

import type { ReactNode } from "react";
import { AuthSessionProvider } from "@/features/auth/useAuthSession";

import "./globals.css";

export const metadata: Metadata = {
  title: "Cuentas Claras",

  description:
    "Tus cuentas personales, organizadas por moneda en tu dispositivo.",
};

type RootLayoutProps = {
  children: ReactNode;
};

export default function RootLayout({ children }: RootLayoutProps) {
  return (
    <html lang="es" suppressHydrationWarning>
      <body>
        <AuthSessionProvider>
          {children}
        </AuthSessionProvider>
      </body>
    </html>
  );
}

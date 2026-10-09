import sys
from pathlib import Path

import yaml

from cuentas_claras.app.main import app

DESTINO = Path(__file__).resolve().parent.parent / "openapi.yaml"


def generar() -> str:
    return yaml.safe_dump(app.openapi(), sort_keys=False, allow_unicode=True)


def main() -> None:
    contenido = generar()
    if "--check" in sys.argv:
        actual = DESTINO.read_text(encoding="utf-8") if DESTINO.exists() else ""
        if actual != contenido:
            print("openapi.yaml está desactualizado.")
            print("Corre: python scripts/exportar_openapi.py")
            sys.exit(1)
        print("openapi.yaml está al día.")
        return
    DESTINO.write_text(contenido, encoding="utf-8", newline="\n")
    print(f"Escrito {DESTINO}")


if __name__ == "__main__":
    main()

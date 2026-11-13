"""Pruebas basadas en propiedades de la división de gastos (INV-07, INV-08).

Llaman a la función de división real con datos al azar y comprueban reglas
que deben cumplirse siempre, no solo en los ejemplos del documento.
"""

from decimal import Decimal
from types import SimpleNamespace

from hypothesis import given, settings
from hypothesis import strategies as st

from cuentas_claras.app.rutas.social import _dividir

NOMBRES = st.lists(
    st.text(alphabet="abcdefghijklmnopqrstuvwxyz", min_size=3, max_size=8),
    min_size=1,
    max_size=8,
    unique=True,
)
# Hasta 10 billones de céntimos: cubre montos grandes.
MONTOS = st.integers(min_value=1, max_value=10**13)


def _participantes(nombres: list[str]) -> list[SimpleNamespace]:
    return [SimpleNamespace(nombre_usuario=n) for n in nombres]


def _marcados(resultado: dict[str, tuple[int, bool]]) -> list[str]:
    return [n for n, (_, residuo) in sorted(resultado.items()) if residuo]


@st.composite
def porcentajes(draw: st.DrawFn, nombres: list[str]) -> dict[str, Decimal]:
    """Porcentajes con dos decimales que suman exactamente 100."""
    cortes = sorted(
        draw(
            st.lists(
                st.integers(0, 10000),
                min_size=len(nombres) - 1,
                max_size=len(nombres) - 1,
            )
        )
    )
    limites = [0, *cortes, 10000]
    centesimas = [limites[i + 1] - limites[i] for i in range(len(nombres))]
    return {n: Decimal(c).scaleb(-2) for n, c in zip(nombres, centesimas, strict=True)}


@settings(max_examples=300)
@given(nombres=NOMBRES, monto=MONTOS)
def test_iguales_suman_el_monto_y_el_residuo_va_por_orden_alfabetico(
    nombres: list[str], monto: int
) -> None:
    resultado = _dividir(monto, "iguales", {}, _participantes(nombres))
    ordenados = sorted(nombres)
    assert sum(parte for parte, _ in resultado.values()) == monto
    assert _marcados(resultado) == ordenados[: monto % len(nombres)]
    base = monto // len(nombres)
    for parte, residuo in resultado.values():
        assert parte == base + (1 if residuo else 0)


@settings(max_examples=300)
@given(data=st.data(), nombres=NOMBRES, monto=MONTOS)
def test_porcentajes_suman_el_monto_sin_perder_ni_inventar_centimos(
    data: st.DataObject, nombres: list[str], monto: int
) -> None:
    valores = data.draw(porcentajes(nombres))
    resultado = _dividir(monto, "porcentajes", valores, _participantes(nombres))
    ordenados = sorted(nombres)
    marcados = _marcados(resultado)
    assert sum(parte for parte, _ in resultado.values()) == monto
    assert marcados == ordenados[: len(marcados)]
    for nombre, (parte, residuo) in resultado.items():
        exacta = Decimal(monto) * valores[nombre] / Decimal(100)
        assert parte == int(exacta) + (1 if residuo else 0)


@settings(max_examples=300)
@given(
    montos=st.dictionaries(
        st.text(alphabet="abcdefghijklmnopqrstuvwxyz", min_size=3, max_size=8),
        st.integers(0, 10**12),
        min_size=1,
        max_size=8,
    )
)
def test_montos_fijos_que_cuadran_se_guardan_tal_cual_sin_residuo(
    montos: dict[str, int],
) -> None:
    total = sum(montos.values())
    valores = {n: Decimal(c).scaleb(-2) for n, c in montos.items()}
    resultado = _dividir(total, "fijos", valores, _participantes(list(montos)))
    assert {n: parte for n, (parte, _) in resultado.items()} == montos
    assert _marcados(resultado) == []


@settings(max_examples=200)
@given(nombres=NOMBRES, monto=MONTOS)
def test_la_division_es_determinista(nombres: list[str], monto: int) -> None:
    primero = _dividir(monto, "iguales", {}, _participantes(nombres))
    segundo = _dividir(monto, "iguales", {}, _participantes(list(reversed(nombres))))
    assert primero == segundo

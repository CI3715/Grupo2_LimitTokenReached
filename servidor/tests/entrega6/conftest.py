from collections.abc import Iterator

import pytest
from qa.entrega6.escenario import EscenarioSocial, crear_escenario


@pytest.fixture
def escenario() -> Iterator[EscenarioSocial]:
    with crear_escenario() as actual:
        yield actual

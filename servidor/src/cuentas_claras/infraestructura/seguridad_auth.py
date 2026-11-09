from datetime import datetime, timedelta, timezone

import jwt
from argon2 import PasswordHasher
from argon2.exceptions import VerificationError
from jwt.exceptions import InvalidTokenError

from cuentas_claras.app.config.config import settings

_password_hasher = PasswordHasher()
_HASH_DUMMY = _password_hasher.hash("clave-dummy-no-utilizable")


def crear_hash(contrasena: str) -> str:
    return _password_hasher.hash(contrasena)


def verificar_contrasena(hash_guardado: str | None, contrasena: str) -> bool:
    hash_a_verificar = hash_guardado or _HASH_DUMMY
    try:
        coincide = _password_hasher.verify(hash_a_verificar, contrasena)
    except VerificationError:
        return False
    return hash_guardado is not None and coincide


def crear_token(usuario_id: str, ahora: datetime | None = None) -> tuple[str, datetime]:
    ahora = ahora or datetime.now(timezone.utc)
    vence = ahora + timedelta(minutes=settings.JWT_TTL_MINUTES)

    token = jwt.encode(
        {
            "sub": usuario_id,
            "iss": settings.JWT_ISSUER,
            "iat": ahora,
            "exp": vence,
        },
        settings.JWT_SECRET.get_secret_value(),
        algorithm="HS256",
    )
    return token, vence


def verificar_token(token: str, ahora: datetime | None = None) -> str:
    ahora = ahora or datetime.now(timezone.utc)
    payload = jwt.decode(
        token,
        settings.JWT_SECRET.get_secret_value(),
        algorithms=["HS256"],
        issuer=settings.JWT_ISSUER,
        options={"verify_exp": False, "verify_iat": False},
    )

    usuario_id = payload.get("sub")
    if not isinstance(usuario_id, str) or not usuario_id:
        raise InvalidTokenError("Token sin usuario.")

    vencimiento = payload.get("exp")
    if isinstance(vencimiento, bool) or not isinstance(vencimiento, (int, float)):
        raise InvalidTokenError("Token sin vencimiento.")
    try:
        vence_en = datetime.fromtimestamp(vencimiento, tz=timezone.utc)
    except (OverflowError, OSError, ValueError):
        raise InvalidTokenError("Vencimiento de token inválido.") from None
    if vence_en <= ahora.astimezone(timezone.utc):
        raise InvalidTokenError("Token vencido.")

    return usuario_id

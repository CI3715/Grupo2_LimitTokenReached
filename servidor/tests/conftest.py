import os
import tempfile
from pathlib import Path

os.environ.setdefault(
    "JWT_SECRET", "only-for-automated-tests-do-not-use-in-deployment-32-bytes"
)
os.environ["JWT_TTL_MINUTES"] = "60"
os.environ.setdefault(
    "DATABASE_PATH",
    str(Path(tempfile.gettempdir()) / f"cuentas-claras-tests-{os.getpid()}.db"),
)

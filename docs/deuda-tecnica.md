# Deuda técnica

## Token de sesión en Android

En Android, la Entrega 5 guarda el token de sesión en
`app_config_dir/session.token`, dentro del directorio privado de la aplicación,
con permisos de lectura/escritura restringidos al proceso. El archivo contiene
solo el token, nunca la contraseña. Esta alternativa se usa porque el backend
de `keyring` no está habilitado para Android en esta implementación.

**Pendiente:** reemplazar este archivo por almacenamiento respaldado por el
keystore del sistema y probar restauración/eliminación de la sesión en Android.

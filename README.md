# Grupo2_LimitTokenReached
Repo del Grupo 2 – Sección 1 para el proyecto del curso CI3715 (sep – dic 2026)

# Papeles de cada parte de la aplicacion de cuentas claras

 *Servidor* : Solo guarda lo que las otras personas necesitan ver: amistades, solicitudes entre usuarios y grupos con datos compartidos. 

 *Aplicacion (Escritorio y Android)* : Puede guardar los datos financieros y personales de la persona en la aplicacion local, jamas en el servidor. Y no los expone sin la debida autorizacion del mismo. 

# Objetivos de la aplicacion

*Registro de los movimientos y cuentas de los clientes*
*Conversion de monedas, exportacion e importacion del libro personal del usuario con los dispositivos*

# Servidor: 

*Para iniciar el servidor de desarrollo en local*
Para poder arrancar el servidor de desarrollo en local debes ejecutar los siguientes comandos: 

1. Crear el entorno virtual que necesitas para ejecutar el servidor de desarrollo de FastAPI: 

python -m venv env 

2. Moverte a la carpeta app (que es donde esa la aplicacion de FastAPI)
cd ~/servidor/src/cuentas_claras/app 


3. Descargar las dependencias necesarias dentro del archivo requirements.txt

pip install -r requirements.txt

4. Ejecutar el comando uvicorn main:app --reload
## Vista Mis cuentas — Entrega 2

La aplicación abre en **Mis cuentas**, con nombre, tipo, moneda y saldo, subtotales separados por moneda y formulario **Nueva cuenta**. La vista de conexión en `/conexion/` comparte el diseño y la navegación de Mis cuentas.

Esta implementaci?n cubre ?nicamente la vista, con datos de muestra y estado temporal de React. Lo creado se pierde al recargar. No incluye backend, persistencia ni llamadas a Tauri. Conexi?n es una secci?n visual deshabilitada.

Consulta el [contrato de integración y las comprobaciones de la vista](docs/qa-mis-cuentas.md).

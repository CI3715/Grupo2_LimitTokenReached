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


# Realizacion de Test en el servidor de FastAPI
    Desde la carpeta servidor se debe probar lo siguiente, con el entorno virtual de venv activo: 
1. Primero ejecuta el comando: 
    ruff check .

    Este comando se encarga de correr los test de Ruff en busca de errores de sintaxis, logica, etc. 

2. Para verificar el formateo correcto del codigo, usamos black, ejecutando el siguiente comando: 

    black --check . 

    Nota: si este test no pasa, corre el comando black . para corregir los archivos y que 
    tenga un formato valido. 

3. Luego ejecuta este comando, para verificar que los tipados estaticos impuestos por pydantic se esten 
    respetando: 

    mypy

4. Luego para finalizar, verificaremos que los imports dentro del programa se esten llamando
    correctamente, para ello usaremos el comando: 

    PYTHONPATH=src lint-imports 

    
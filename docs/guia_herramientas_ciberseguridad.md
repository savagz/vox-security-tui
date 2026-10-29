# Catálogo de herramientas de ciberseguridad: casos de uso y ejemplos

> **Uso autorizado:** los comandos de reconocimiento, escaneo, auditoría
> y pruebas deben ejecutarse únicamente sobre sistemas propios,
> laboratorios o activos para los que exista autorización explícita. Los
> ejemplos usan `192.0.2.10` (rango reservado para documentación),
> `lab.local` y servicios locales como referencias ilustrativas.
> Sustituye esos valores solo por objetivos autorizados.
>
> **Antes de empezar:** comprueba que la herramienta esté instalada
> (`comando --help` o `man comando`), adapta las opciones a la versión
> instalada y guarda los resultados en un directorio de trabajo. Algunos
> comandos requieren `sudo`/privilegios de administrador.

## 1. Reconocimiento

### 1.1 `nmap` --- descubrimiento de hosts y puertos

**Caso:** inventariar los puertos TCP de un servidor de laboratorio.

``` bash
nmap -sV -sC -oN nmap-lab.txt 192.0.2.10
```

-   `-sV`: intenta identificar versiones de servicios.
-   `-sC`: ejecuta scripts predeterminados de Nmap.
-   `-oN`: guarda el resultado en texto normal.
-   Para descubrir hosts de una subred autorizada:
    `nmap -sn 192.0.2.0/24`.

**Resultado esperado:** lista de puertos, estados y servicios
detectados. Los scripts predeterminados también pueden generar tráfico
adicional; revísalos antes de usarlos en entornos sensibles.

### 1.2 `dnsrecon` --- reconocimiento DNS

**Caso:** revisar registros DNS publicados para un dominio propio.

``` bash
dnsrecon -d ejemplo.com -t std
```

Para consultar una zona de laboratorio concreta mediante un servidor
DNS:

``` bash
dnsrecon -d lab.local -n 192.0.2.53 -t std
```

**Resultado esperado:** registros DNS que la herramienta consiga
consultar, como A, AAAA, MX, NS y TXT. La disponibilidad depende de la
configuración DNS.

### 1.3 `theHarvester` --- recopilación de información pública

**Caso:** recopilar fuentes públicas relacionadas con el dominio de una
organización propia.

``` bash
theHarvester -d ejemplo.com -b duckduckgo
```

La sintaxis de proveedores y opciones cambia según la versión. Consulta
`theHarvester -h` para elegir una fuente compatible.

**Resultado esperado:** posibles nombres, correos y subdominios
encontrados en las fuentes seleccionadas. Los datos pueden estar
incompletos o desactualizados; valida cada hallazgo.

### 1.4 `whois` --- información de registro de dominios/IP

**Caso:** consultar los datos de registro disponibles públicamente.

``` bash
whois ejemplo.com
whois 192.0.2.10
```

**Resultado esperado:** información de registrador, fechas, servidores
de nombres o asignación de direcciones, si el registro la publica. Los
servicios de privacidad pueden ocultar datos.

### 1.5 `amass` --- mapeo de superficie DNS

**Caso:** enumerar pasivamente subdominios de un dominio autorizado.

``` bash
amass enum -passive -d ejemplo.com
```

**Resultado esperado:** nombres de subdominio identificados por fuentes
pasivas. La sintaxis exacta varía entre versiones mayores; verifica
`amass enum -h`.

### 1.6 `fierce` --- búsqueda de hosts y subdominios DNS

**Caso:** examinar la exposición DNS de un dominio propio.

``` bash
fierce --domain ejemplo.com
```

**Resultado esperado:** nombres y direcciones que la herramienta logra
resolver. Úsala solo con dominios autorizados y revisa `fierce --help`
si la versión instalada usa opciones distintas.

## 2. Escaneo de red

### 2.1 `masscan` --- escaneo rápido de puertos

**Caso:** comprobar un puerto concreto en un host de laboratorio.

``` bash
sudo masscan 192.0.2.10 -p80,443 --rate 100
```

-   `-p`: puertos a comprobar.
-   `--rate 100`: limita el ritmo aproximado de envío de paquetes.

**Precaución:** Masscan puede generar mucho tráfico. Empieza con tasas
bajas y alcances pequeños; no escanees rangos públicos ni redes
compartidas sin autorización.

### 2.2 `arp-scan` --- descubrimiento de equipos en la LAN

**Caso:** identificar equipos IPv4 visibles en una red local a la que
tienes acceso.

``` bash
sudo arp-scan --localnet
```

Para especificar una interfaz:

``` bash
sudo arp-scan --interface=eth0 --localnet
```

**Resultado esperado:** direcciones IP y MAC de los dispositivos que
respondan a ARP en el segmento local. No atraviesa routers como un
escaneo IP convencional.

### 2.3 `netdiscover` --- descubrimiento ARP

**Caso:** localizar equipos en la red local durante un inventario
autorizado.

``` bash
sudo netdiscover -i eth0 -r 192.168.1.0/24
```

**Resultado esperado:** IP, MAC y, cuando se puede inferir, fabricante.
Ajusta la interfaz y la subred a tu laboratorio.

### 2.4 `hping3` --- diagnóstico y generación controlada de paquetes

**Caso:** comprobar si un servidor de laboratorio responde a solicitudes
TCP SYN al puerto 443.

``` bash
sudo hping3 -S -p 443 -c 4 192.0.2.10
```

-   `-S`: establece el indicador SYN.
-   `-p 443`: puerto de destino.
-   `-c 4`: envía cuatro paquetes.

**Precaución:** úsalo para diagnóstico controlado, no para inundación ni
degradación de servicios. La respuesta no sustituye un escaneo completo.

## 3. Auditoría de aplicaciones web

### 3.1 `nikto` --- comprobaciones comunes de servidores web

**Caso:** revisar un servidor HTTP de pruebas.

``` bash
nikto -h http://192.0.2.10
```

**Resultado esperado:** configuraciones inseguras, archivos conocidos y
versiones potencialmente problemáticas. Algunos hallazgos son falsos
positivos; confirma manualmente y limita el impacto en producción.

### 3.2 `gobuster` --- enumeración de rutas web

**Caso:** descubrir directorios comunes en una aplicación de
laboratorio.

``` bash
gobuster dir -u http://192.0.2.10 -w /usr/share/wordlists/dirb/common.txt
```

**Resultado esperado:** rutas que respondan con códigos HTTP relevantes.
Usa listas pequeñas al comenzar y respeta límites de velocidad del
entorno.

### 3.3 `ffuf` --- fuzzing de rutas y parámetros

**Caso:** buscar rutas usando una lista de palabras en una aplicación
propia.

``` bash
ffuf -u http://192.0.2.10/FUZZ -w /usr/share/wordlists/dirb/common.txt -fc 404
```

-   `FUZZ` es el marcador que se reemplaza con cada palabra.
-   `-fc 404` filtra respuestas HTTP 404.

**Resultado esperado:** rutas candidatas. Las respuestas dinámicas
pueden producir falsos positivos; compara tamaño, código y contenido.

### 3.4 `sqlmap` --- pruebas de inyección SQL

**Caso:** comprobar una aplicación deliberadamente vulnerable instalada
en tu propio equipo, por ejemplo DVWA en un laboratorio local.

``` bash
sqlmap -u 'http://127.0.0.1/vulnerable.php?id=1' --batch --risk=1 --level=1
```

**Alcance:** el comando realiza pruebas de detección limitadas sobre la
URL indicada. No añadas opciones de extracción de datos, escritura o
acceso al sistema salvo que formen parte de una prueba aprobada y
controlada. No apuntes a sitios ajenos.

### 3.5 `whatweb` --- identificación de tecnologías web

**Caso:** identificar tecnologías visibles en un servidor propio.

``` bash
whatweb http://192.0.2.10
```

Para un informe detallado:

``` bash
whatweb -v http://192.0.2.10
```

**Resultado esperado:** indicios de servidor web, CMS, bibliotecas y
cabeceras. La identificación es heurística y no demuestra por sí sola
que una versión sea vulnerable.

### 3.6 `nuclei` --- comprobaciones mediante plantillas

**Caso:** ejecutar plantillas de detección sobre una aplicación de
pruebas.

``` bash
nuclei -u http://192.0.2.10 -severity info,low,medium
```

Para listar las plantillas disponibles y entender sus etiquetas:

``` bash
nuclei -tl
```

**Precaución:** revisa las plantillas y sus efectos antes de usarlas.
Evita plantillas intrusivas o de explotación en producción y mantén las
plantillas actualizadas desde una fuente confiable.

## 4. Auditoría de contraseñas (solo hashes propios o laboratorio)

### 4.1 `John the Ripper` --- auditoría de fortaleza de hashes

**Caso:** comprobar un archivo de hashes de prueba obtenido
legítimamente.

``` bash
john --wordlist=/usr/share/wordlists/rockyou.txt hashes-lab.txt
john --show hashes-lab.txt
```

**Resultado esperado:** contraseñas que coincidan con los hashes y que
la lista permita encontrar. No uses hashes de terceros sin autorización.
La ruta de la lista varía según la distribución.

### 4.2 `hashcat` --- recuperación de contraseñas desde hashes

**Caso:** auditar un hash de prueba cuyo tipo se conoce. El ejemplo usa
modo 0 (MD5) y un archivo de laboratorio.

``` bash
hashcat -m 0 -a 0 hashes-md5-lab.txt /ruta/a/wordlist.txt
hashcat -m 0 --show hashes-md5-lab.txt
```

-   `-m 0`: hash MD5 sin sal.
-   `-a 0`: ataque de diccionario.

**Precaución:** MD5 no es adecuado para almacenar contraseñas. Para
evaluar sistemas reales, usa hashes exportados con permiso y protege los
resultados.

### 4.3 `hashid` --- identificación probable de tipos de hash

**Caso:** obtener posibles formatos para un hash de prueba.

``` bash
hashid '5f4dcc3b5aa765d61d8327deb882cf99'
```

**Resultado esperado:** una lista de algoritmos candidatos. El formato
no se puede confirmar solo por longitud o apariencia; contrástalo con el
sistema que generó el hash.

### 4.4 `hydra` --- prueba autorizada de autenticación

**Caso:** validar controles de bloqueo y limitación de intentos en un
servicio de laboratorio con una cuenta de prueba. Usa únicamente
credenciales ficticias y un servidor que controles.

``` bash
hydra -l auditor_lab -P ./passwords-lab.txt -t 1 ssh://192.0.2.10
```

**Precaución importante:** incluso una prueba pequeña puede bloquear
cuentas o activar alertas. Coordina una ventana de pruebas, limita los
intentos y detente si el servicio muestra efectos inesperados. No uses
listas de credenciales reales ni cuentas ajenas.

## 5. Redes inalámbricas

### 5.1 `aircrack-ng` --- auditoría de capturas Wi-Fi propias

**Caso:** analizar una captura de auditoría de tu propia red, obtenida
con autorización.

``` bash
aircrack-ng captura-lab.cap
```

Si la evaluación aprobada contempla una lista de contraseñas de
laboratorio:

``` bash
aircrack-ng -w passwords-lab.txt captura-lab.cap
```

**Resultado esperado:** análisis de la captura y, si se cumplen las
condiciones necesarias, posible coincidencia con una clave de la lista.
Una captura por sí sola no garantiza que la clave pueda recuperarse. No
captures ni ataques redes ajenas.

### 5.2 `bettercap` --- inspección de red en laboratorio

**Caso:** iniciar Bettercap en una interfaz dedicada de laboratorio y
consultar los comandos disponibles.

``` bash
sudo bettercap -iface eth0
```

En la consola interactiva:

``` text
help
net.probe on
net.show
```

`net.probe` puede generar tráfico de descubrimiento en la red. Úsalo
solo en un segmento aislado o expresamente autorizado y desactívalo al
terminar (`net.probe off`, si está activo).

## 6. Hardening y detección de rootkits

### 6.1 `lynis` --- auditoría de configuración de seguridad

**Caso:** ejecutar una auditoría local del sistema Linux.

``` bash
sudo lynis audit system
```

**Resultado esperado:** hallazgos y recomendaciones sobre configuración,
permisos, servicios y controles de seguridad. Revisa las recomendaciones
antes de modificar el sistema; no todas aplican a todos los entornos.

### 6.2 `chkrootkit` --- búsqueda de indicadores de rootkits

**Caso:** realizar una comprobación básica del equipo que administras.

``` bash
sudo chkrootkit
```

**Resultado esperado:** pruebas e indicadores que requieren revisión.
Puede haber falsos positivos y una salida limpia no garantiza que el
sistema esté libre de compromiso.

### 6.3 `rkhunter` --- comprobaciones de rootkits y cambios sospechosos

**Caso:** actualizar la información de la herramienta y ejecutar una
revisión.

``` bash
sudo rkhunter --update
sudo rkhunter --check
```

En algunas distribuciones, la actualización puede gestionarse mediante
el gestor de paquetes. Lee los avisos y verifica cambios legítimos antes
de marcarlos como excepciones.

### 6.4 `clamscan` --- análisis antivirus con ClamAV

**Caso:** analizar una carpeta de archivos descargados.

``` bash
clamscan --recursive --infected ~/Descargas
```

Para guardar el resultado:

``` bash
clamscan --recursive --infected --log=clamscan.log ~/Descargas
```

**Resultado esperado:** archivos que coincidan con firmas conocidas.
Mantén las firmas actualizadas y recuerda que ningún antivirus detecta
todas las amenazas.

## 7. Análisis forense y metadatos

### 7.1 `fls` --- listar archivos de una imagen forense

**Caso:** listar entradas de una imagen de disco adquirida legalmente
con The Sleuth Kit.

``` bash
fls -r -f ext4 imagen-disco.dd
```

El tipo de sistema de archivos (`-f`) debe coincidir con la imagen; si
no lo conoces, consulta `fls -h` y las herramientas de identificación.
Trabaja sobre una copia y conserva intacta la evidencia original.

### 7.2 `foremost` --- recuperación de archivos por firmas

**Caso:** recuperar archivos de una imagen de laboratorio en un
directorio de salida.

``` bash
mkdir -p recuperados
foremost -i imagen-disco.dd -o recuperados
```

**Resultado esperado:** archivos recuperados por sus firmas, cuando sea
posible. Puede perder nombres, rutas o metadatos y producir fragmentos
incompletos. Documenta hashes y cadena de custodia en investigaciones
formales.

### 7.3 `binwalk` --- inspección de firmware y archivos binarios

**Caso:** localizar firmas de archivos y estructuras embebidas en
firmware propio.

``` bash
binwalk firmware-lab.bin
```

Para extraer contenido reconocido en una copia de laboratorio:

``` bash
binwalk -e firmware-lab.bin
```

**Precaución:** inspecciona el contenido extraído antes de abrirlo o
ejecutarlo. El firmware puede contener datos sensibles; trabaja con una
copia autorizada.

### 7.4 `exiftool` --- lectura de metadatos

**Caso:** revisar metadatos de una imagen propia antes de publicarla.

``` bash
exiftool foto.jpg
```

Para eliminar metadatos de una copia, conservando el original:

``` bash
cp foto.jpg foto-sin-metadatos.jpg
exiftool -all= foto-sin-metadatos.jpg
```

**Resultado esperado:** datos como modelo de cámara, fechas o
coordenadas, si están presentes. Verifica el archivo resultante y no
modifiques la evidencia original.

## 8. Captura y análisis de tráfico

### 8.1 `tcpdump` --- captura de paquetes

**Caso:** capturar tráfico DNS durante un diagnóstico autorizado.

``` bash
sudo tcpdump -i eth0 -nn 'port 53' -c 100 -w dns-lab.pcap
```

-   `-i eth0`: interfaz.
-   `-nn`: evita resolver nombres y puertos.
-   `-c 100`: termina tras 100 paquetes.
-   `-w`: guarda la captura.

Para leer una captura existente:

``` bash
tcpdump -nn -r dns-lab.pcap
```

**Privacidad:** las capturas pueden contener credenciales, tokens o
datos personales. Limita el alcance, protege el archivo y elimínalo
según la política de retención.

### 8.2 `tshark` --- análisis de paquetes desde terminal

**Caso:** extraer consultas DNS de una captura existente.

``` bash
tshark -r dns-lab.pcap -Y dns
```

Para mostrar campos concretos:

``` bash
tshark -r dns-lab.pcap -Y dns -T fields -e frame.time -e ip.src -e dns.qry.name
```

**Resultado esperado:** paquetes que coincidan con el filtro de
visualización. Consulta `tshark -G fields` para comprobar los nombres de
campos disponibles en tu versión.

### 8.3 `ngrep` --- búsqueda de patrones en tráfico

**Caso:** observar mensajes DNS en una interfaz de laboratorio.

``` bash
sudo ngrep -d eth0 -W byline '' 'port 53'
```

También se puede analizar una captura guardada:

``` bash
ngrep -I dns-lab.pcap -W byline '' 'port 53'
```

**Precaución:** `ngrep` puede mostrar contenido de aplicaciones en texto
claro. Evita capturar tráfico de usuarios que no hayan autorizado la
prueba.

### 8.4 `suricata` --- detección de amenazas en tráfico

**Caso:** procesar una captura de laboratorio con las reglas
configuradas localmente.

``` bash
sudo suricata -r dns-lab.pcap -l ./suricata-logs
```

**Resultado esperado:** alertas y registros en el directorio indicado,
según las reglas disponibles. Para monitorización en vivo, revisa la
configuración de interfaces, reglas y modo de ejecución de tu
distribución antes de habilitarlo.

## 9. Flujo de trabajo sugerido para un laboratorio

1.  **Definir el alcance:** documenta IP, dominio, horario, responsable
    y pruebas permitidas.
2.  **Reconocer:** empieza con `whois`, `dnsrecon`, `theHarvester` y
    `amass` sobre activos autorizados.
3.  **Inventariar servicios:** usa `nmap`; recurre a `masscan` solo si
    necesitas velocidad y tienes control del alcance.
4.  **Revisar aplicaciones:** combina `whatweb`, `nikto`, `gobuster`,
    `ffuf` y `nuclei` con límites prudentes.
5.  **Validar manualmente:** verifica hallazgos y evita convertir una
    detección potencial en una afirmación de vulnerabilidad sin
    evidencia.
6.  **Auditar el sistema:** utiliza `lynis`, `chkrootkit`, `rkhunter` y
    `clamscan`.
7.  **Analizar evidencia y tráfico:** conserva copias, calcula hashes de
    evidencia cuando corresponda y limita el acceso a capturas.
8.  **Informar:** registra comando, fecha, alcance, resultado,
    limitaciones, evidencia y acciones correctivas.

## 10. Buenas prácticas

-   No ejecutes pruebas sobre sistemas de terceros sin permiso
    explícito.
-   Usa un laboratorio aislado para pruebas de contraseñas, Wi-Fi,
    fuzzing y vulnerabilidades.
-   No compartas capturas, hashes, informes o archivos forenses que
    contengan datos sensibles.
-   Registra versiones (`nmap --version`, `sqlmap --version`, etc.) para
    que el trabajo sea reproducible.
-   Revisa la documentación de la versión instalada: las opciones pueden
    cambiar entre distribuciones y versiones.
-   Trata los resultados automáticos como indicios que deben validarse,
    no como prueba concluyente.

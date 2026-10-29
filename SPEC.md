# SecurityTUI — Análisis de paquetes

App TUI para Linux que reúne herramientas de hacking ético y análisis de seguridad, para **auditoría autorizada y uso defensivo**. Abajo, las herramientas candidatas agrupadas por categoría con su nombre de paquete en Debian/Kali, Arch y Fedora, y su licencia.

> Nota: en Debian/Ubuntu muchas viven en `kali-linux-*` metapaquetes o en los repos de Kali/Parrot; en Arch, buena parte están en AUR o en el repo **BlackArch**; en Fedora, varias en el repo oficial y otras en COPR/GitHub.

---

## 1. Reconocimiento (OSINT / info gathering)

| Herramienta | Debian/Kali | Arch | Fedora | Licencia |
|---|---|---|---|---|
| nmap | `nmap` | `nmap` | `nmap` | NPSL (tipo GPL) |
| dnsrecon | `dnsrecon` | AUR `dnsrecon` | `dnsrecon` | GPL-2.0 |
| whois | `whois` | `whois` | `whois` | GPL-2.0 |
| amass | `amass` | `amass` (AUR) | GitHub/COPR | Apache-2.0 |
| recon-ng | `recon-ng` | BlackArch | GitHub | GPL-3.0 |
| fierce | `fierce` | AUR | GitHub | GPL-3.0 |

## 2. Escaneo de red

| Herramienta | Debian/Kali | Arch | Fedora | Licencia |
|---|---|---|---|---|
| nmap / ncat | `nmap` | `nmap` | `nmap` | NPSL |
| masscan | `masscan` | `masscan` | `masscan` | AGPL-3.0 |
| netdiscover | `netdiscover` | BlackArch | GitHub | GPL-3.0 |
| arp-scan | `arp-scan` | `arp-scan` (AUR) | `arp-scan` | GPL-3.0 |
| rustscan | GitHub/`.deb` | AUR `rustscan` | GitHub | GPL-3.0 |
| hping3 | `hping3` | AUR `hping` | `hping3` | GPL-2.0 |

## 3. Web

| Herramienta | Debian/Kali | Arch | Fedora | Licencia |
|---|---|---|---|---|
| nikto | `nikto` | AUR `nikto` | `nikto` | GPL-2.0 |
| gobuster | `gobuster` | `gobuster` | `gobuster` | Apache-2.0/MIT |
| ffuf | `ffuf` | `ffuf` | GitHub/COPR | MIT |
| sqlmap | `sqlmap` | `sqlmap` | `sqlmap` | GPL-2.0 |
| wpscan | `wpscan` | AUR `wpscan` | GitHub (gem) | propietaria (gratis no comercial) |
| whatweb | `whatweb` | BlackArch | GitHub | GPL-3.0 |
| nuclei | GitHub/`.deb` | `nuclei` (AUR) | GitHub | MIT |
| zaproxy (OWASP ZAP) | `zaproxy` | AUR `zaproxy` | Flatpak | Apache-2.0 |

## 4. Contraseñas y hashes

| Herramienta | Debian/Kali | Arch | Fedora | Licencia |
|---|---|---|---|---|
| john (John the Ripper) | `john` | `john` | `john` | GPL-2.0 |
| hashcat | `hashcat` | `hashcat` | `hashcat` | MIT |
| hydra | `hydra` | AUR `hydra` | `hydra` | AGPL-3.0 |
| medusa | `medusa` | BlackArch | `medusa` | GPL-2.0 |
| hashid | `hashid` | AUR | GitHub | GPL-3.0 |
| crunch | `crunch` | AUR | GitHub | GPL-2.0 |

## 5. Wireless

| Herramienta | Debian/Kali | Arch | Fedora | Licencia |
|---|---|---|---|---|
| aircrack-ng | `aircrack-ng` | `aircrack-ng` | `aircrack-ng` | GPL-2.0 |
| kismet | `kismet` | AUR `kismet` | `kismet` | GPL-2.0 |
| reaver | `reaver` | BlackArch | GitHub | GPL-2.0 |
| wifite | `wifite` | AUR `wifite` | GitHub | GPL-2.0 |
| hcxtools / hcxdumptool | `hcxtools` | AUR | `hcxtools` | MIT |
| bettercap | `bettercap` | `bettercap` | GitHub | GPL-3.0 |

## 6. Análisis de sistema / hardening

| Herramienta | Debian/Kali | Arch | Fedora | Licencia |
|---|---|---|---|---|
| lynis | `lynis` | `lynis` | `lynis` | GPL-3.0 |
| chkrootkit | `chkrootkit` | AUR | `chkrootkit` | BSD |
| rkhunter | `rkhunter` | AUR | `rkhunter` | GPL-2.0 |
| openscap / scap-security-guide | `libopenscap8` | AUR | `openscap-scanner` | LGPL-2.1 |
| clamav | `clamav` | `clamav` | `clamav` | GPL-2.0 |

## 7. Forense

| Herramienta | Debian/Kali | Arch | Fedora | Licencia |
|---|---|---|---|---|
| sleuthkit | `sleuthkit` | `sleuthkit` | `sleuthkit` | CPL/IPL/GPL |
| autopsy | `autopsy` | AUR | GitHub | Apache-2.0 |
| volatility3 | `volatility3` | AUR | GitHub | GPL-2.0 (Vol Foundation) |
| foremost | `foremost` | AUR | `foremost` | dominio público |
| binwalk | `binwalk` | `binwalk` | `binwalk` | MIT |
| exiftool | `libimage-exiftool-perl` | `perl-image-exiftool` | `perl-Image-ExifTool` | Perl (GPL/Artistic) |

## 8. Monitoreo / análisis de tráfico

| Herramienta | Debian/Kali | Arch | Fedora | Licencia |
|---|---|---|---|---|
| wireshark / tshark | `wireshark` / `tshark` | `wireshark-cli` | `wireshark-cli` | GPL-2.0 |
| tcpdump | `tcpdump` | `tcpdump` | `tcpdump` | BSD |
| ngrep | `ngrep` | AUR | `ngrep` | BSD-like |
| iftop | `iftop` | `iftop` | `iftop` | GPL-2.0 |
| ntopng | `ntopng` | AUR | COPR | GPL-3.0 |
| suricata | `suricata` | `suricata` | `suricata` | GPL-2.0 |

---

## Librerías para construir la TUI

| Stack | Librería | Pros | Contras |
|---|---|---|---|
| **Python** | **Textual** + Rich | Desarrollo rápido, widgets ricos, CSS-like, async; ideal para lanzar/parsear CLIs | Overhead de runtime Python; empaquetado más pesado |
| **Rust** | **Ratatui** (+ crossterm) | Binario único, muy rápido, sin dependencias de runtime | Más código boilerplate, curva más alta |
| **Go** | **Bubble Tea** (+ Lipgloss/Bubbles) | Binario único, modelo Elm limpio, buen ecosistema | Menos widgets de alto nivel listos |

### Recomendación

**Textual (Python)** para el MVP. La app es sobre todo un **orquestador**: lanza binarios externos (`nmap`, `john`, etc.), parsea su salida y la presenta. Python hace eso con mínima fricción (`asyncio`/`subprocess`), Textual ya trae tablas, logs, árboles y pestañas, e itera rápido.

Si luego pesa la distribución (binario único, arranque instantáneo), migrar el front a **Ratatui (Rust)** o **Bubble Tea (Go)**; la lógica de orquestación es portable porque las herramientas son procesos externos.

---

## Consideraciones

- **Permisos**: muchas herramientas (captura wireless, raw sockets) requieren `root`/capabilities. Gestionar `sudo`/`CAP_NET_RAW` con cuidado.
- **Licencias mixtas**: GPL/AGPL/propietaria (wpscan). Como la TUI solo **invoca** los binarios (no los enlaza), se evita contaminación de licencia, pero documentar cada una.
- **Empaquetado**: no reempaquetar las herramientas; depender de los repos de la distro o detectar su presencia y guiar la instalación.
- **Ámbito legal**: incluir aviso de uso autorizado y registro de actividad (auditoría).

# Tareas por hacer

## Instalación de amass y nuclei en Ubuntu (vía binarios de release de GitHub)

**Problema (verificado en Ubuntu 24.04):** `amass` y `nuclei` no están en los
repos de Ubuntu (solo en Kali), así que la instalación por apt (tecla `I`) falla.
La vía alternativa actual (`go install`, tecla `⇧A`) requiere tener Go instalado,
que tampoco viene por defecto, y compila proyectos grandes (lento).

**Enfoque elegido (opción 1):** añadir en la app una vía de instalación que
descargue el binario precompilado del release oficial de GitHub. Sin compilar ni
Go; funciona en cualquier Linux x86_64.

**Notas de implementación (pendiente):**
- Ambos publican binarios en GitHub Releases:
  - nuclei: `projectdiscovery/nuclei`
  - amass: `owasp-amass/amass`
- Implica extender el modelo de instalación del catálogo (`src/tools/mod.rs`,
  `src/tools/catalog.rs`): hoy `AltKind` solo cubre `Go` y `Copr`; habría que
  agregar una variante tipo `GithubRelease { repo, asset_pattern }` y su comando
  (descargar tar.gz/zip, verificar, extraer el binario a `/usr/local/bin`).
- Resolver la última versión del release y el nombre del asset por arquitectura
  (x86_64-linux). Considerar checksum si el release lo publica.
- Encaja con el flujo existente de `alt_install_command` / `run_install`.

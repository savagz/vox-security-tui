use serde::{Deserialize, Serialize};

/// Idioma activo de la interfaz. Se guarda en `Config.ui.language` y se puede
/// cambiar en caliente desde el overlay de configuración, sin reiniciar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    #[default]
    Es,
    En,
}

impl Lang {
    pub fn toggle(self) -> Self {
        match self {
            Lang::Es => Lang::En,
            Lang::En => Lang::Es,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Lang::Es => "ES",
            Lang::En => "EN",
        }
    }
}

/// Catálogo de textos de la interfaz. Cada variante es un punto de texto visible
/// en la TUI; `t()` devuelve la cadena (o plantilla `{}`) en el idioma activo.
/// Deliberadamente plano (sin namespaces): un enum único es más fácil de grepear.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    // -- común, reusado entre modales --
    Confirm,
    Cancel,
    Save,
    Field,
    Edit,

    // -- header / sidebar --
    HeaderTitle,
    SidebarConfig,
    SidebarHelp,

    // -- tabs --
    TabOverview,
    TabCatalog,
    TabScan,
    TabLogs,
    TabSystemCheck,

    // -- help bar --
    HbNotice,
    HbView,
    HbJump,
    HbNavigate,
    HbFilter,
    HbInfo,
    HbSelect,
    HbRun,
    HbRerun,
    HbRefresh,
    HbRecheck,
    HbInstall,
    HbInstallAlt,
    HbToolHelp,
    HbClear,
    HbConfig,
    HbQuit,

    // -- help modal --
    HelpTitle,
    HelpSectionGlobal,
    HelpSectionCatalog,
    HelpSectionScan,
    HelpSectionLogs,
    HelpSectionSystemCheck,
    HelpTopBottom,
    HelpClose,

    // -- categorías de herramientas --
    CatRecon,
    CatNetScan,
    CatWeb,
    CatPasswords,
    CatWireless,
    CatHardening,
    CatForensics,
    CatTraffic,

    // -- overview --
    OverviewGaugeCpu,
    OverviewGaugeRam,
    OverviewGaugeDisk,
    OverviewLabelStorage,
    OverviewLabelUptime,
    OverviewToolsTitle,
    OverviewToolsCountTemplate,
    OverviewCategoriesTitle,
    OverviewLegalTitle,
    OverviewLegalBody,

    // -- catálogo --
    CatalogFilterLabel,
    CatalogDetailTemplate,
    ColInstalled,
    ColName,
    ColCategory,
    ColPackage,
    ColLicense,
    ColStatus,
    StatusInstalled,
    StatusMissing,

    // -- scan --
    ScanTitle,
    ScanNoToolSelected,
    ScanIdleTemplate,
    ScanRunningTemplate,
    ScanFinishedOkTemplate,
    ScanFinishedErrTemplate,
    ScanOutputEmpty,
    ScanWaitingOutput,
    ScanPickFromCatalog,
    ScanNotInstalledTemplate,

    // -- logs (historial de corridas) --
    LogsEmpty,
    LogsDetailTemplate,
    LogRunningTag,
    LogOkTag,
    LogErrTag,

    // -- formulario de corrida --
    FormRunTitleTemplate,
    FormLabelTemplate,
    FormLabelTarget,
    FormLabelArgs,
    FormNextField,
    FormTemplateHint,
    FormErrNoCommand,
    FormDestructiveWarning,
    FormDestructiveConfirm,

    // -- info popup --
    InfoTitleTemplate,
    InfoHint,
    InfoFieldCategory,
    InfoFieldLicense,
    InfoFieldBinary,
    InfoFieldStatus,
    InfoFieldPurpose,
    InfoFieldInstall,
    InfoFieldTemplates,
    InfoFieldGo,
    InfoFieldCopr,

    // -- overlay de configuración --
    ConfigFormTitle,
    ConfigFieldRefreshLabel,
    ConfigFieldRefreshHint,
    ConfigFieldTimeoutLabel,
    ConfigFieldTimeoutHint,
    ConfigFieldLanguageLabel,
    ConfigFieldLanguageHint,
    ConfigErrInvalidTemplate,
    ConfigErrZeroTemplate,
    ConfigErrCreateDirTemplate,
    ConfigErrSerialize,
    ConfigErrWriteTemplate,
    ConfigErrNoUserDir,

    // -- system check / deps --
    SetupGateTitle,
    SetupPanelTitle,
    SetupStatusAllOk,
    SetupStatusActionRequiredWithPmTemplate,
    SetupStatusActionRequired,
    SetupStripeAllOk,
    SetupStripeGateBlocking,
    SetupStripePanelBlocking,
    SetupKeyChoose,
    SetupKeyRecheck,
    SetupKeyContinueAnyway,
    SetupKeyExit,

    DepPackageManagerLabel,
    DepPackageManagerOkTemplate,
    DepPackageManagerMissing,
    DepToolsLabel,
    DepToolsOkTemplate,
    DepToolsNoneTemplate,
    DepPrivilegeLabel,
    DepPrivilegeRoot,
    DepPrivilegeUser,
    DepFixturesLabel,
    DepFixturesOn,
    DepFixturesOff,

    // -- main / toasts --
    MainFixturesSimulated,
    MainPressEnterToReturn,
    MainFixDone,
    MainFixError,
    MainFixExecErrTemplate,
    MainNoInstallCommand,
    MainAltInstallTemplate,
    MainNoAltInstall,
    MainScanStartedTemplate,
    MainSentToScanTemplate,
    MainRerunNeedsConfirm,
    MainConfigLoadErrTemplate,
    MainConfigSaveErrTemplate,
    MainConfigSavedTemplate,
}

pub fn t(lang: Lang, key: Key) -> &'static str {
    let (es, en) = pair(key);
    match lang {
        Lang::Es => es,
        Lang::En => en,
    }
}

fn pair(key: Key) -> (&'static str, &'static str) {
    use Key::*;
    match key {
        Confirm => ("confirmar", "confirm"),
        Cancel => ("cancelar", "cancel"),
        Save => ("guardar", "save"),
        Field => ("campo", "field"),
        Edit => ("editar", "edit"),

        HeaderTitle => ("voxsecurity · auditoría de seguridad", "voxsecurity · security audit"),
        SidebarConfig => ("config", "config"),
        SidebarHelp => ("ayuda", "help"),

        TabOverview => ("Resumen", "Overview"),
        TabCatalog => ("Catálogo", "Catalog"),
        TabScan => ("Ejecutar", "Scan"),
        TabLogs => ("Historial", "History"),
        TabSystemCheck => ("Verificación", "System Check"),

        HbNotice => (" AVISO ", " NOTICE "),
        HbView => ("ver", "view"),
        HbJump => ("saltar", "jump"),
        HbNavigate => ("navegar", "navigate"),
        HbFilter => ("filtrar", "filter"),
        HbInfo => ("info", "info"),
        HbSelect => ("elegir para ejecutar", "select to run"),
        HbRun => ("ejecutar", "run"),
        HbRerun => ("repetir", "re-run"),
        HbRefresh => ("refrescar", "refresh"),
        HbRecheck => ("re-verificar", "re-check"),
        HbInstall => ("instalar", "install"),
        HbInstallAlt => ("instalar go/copr", "install go/copr"),
        HbToolHelp => ("ayuda (--help)", "help (--help)"),
        HbClear => ("limpiar", "clear"),
        HbConfig => ("config", "config"),
        HbQuit => ("salir", "quit"),

        HelpTitle => ("Ayuda · atajos", "Help · shortcuts"),
        HelpSectionGlobal => ("Global", "Global"),
        HelpSectionCatalog => ("Catálogo", "Catalog"),
        HelpSectionScan => ("Ejecutar", "Scan"),
        HelpSectionLogs => ("Historial", "History"),
        HelpSectionSystemCheck => ("Verificación", "System Check"),
        HelpTopBottom => ("inicio / fin", "top / bottom"),
        HelpClose => ("cualquier tecla cierra esta ayuda", "any key closes this help"),

        CatRecon => ("Reconocimiento", "Reconnaissance"),
        CatNetScan => ("Escaneo de red", "Network scanning"),
        CatWeb => ("Web", "Web"),
        CatPasswords => ("Contraseñas", "Passwords"),
        CatWireless => ("Wireless", "Wireless"),
        CatHardening => ("Hardening", "Hardening"),
        CatForensics => ("Forense", "Forensics"),
        CatTraffic => ("Tráfico", "Traffic"),

        OverviewGaugeCpu => ("cpu del host", "host cpu"),
        OverviewGaugeRam => ("ram del host", "host ram"),
        OverviewGaugeDisk => ("disco del host", "host disk"),
        OverviewLabelStorage => ("ALMACENAM. ", "STORAGE    "),
        OverviewLabelUptime => ("EN LÍNEA   ", "UPTIME     "),
        OverviewToolsTitle => ("Herramientas", "Tools"),
        OverviewToolsCountTemplate => ("{} de {} instaladas", "{} of {} installed"),
        OverviewCategoriesTitle => ("Por categoría", "By category"),
        OverviewLegalTitle => ("Uso autorizado", "Authorized use"),
        OverviewLegalBody => (
            "Solo para auditoría autorizada y uso defensivo. El uso sin permiso expreso del objetivo puede ser ilegal. Toda actividad queda registrada.",
            "For authorized auditing and defensive use only. Use without the target's express permission may be illegal. All activity is logged.",
        ),

        CatalogFilterLabel => (" FILTRO ", " FILTER "),
        CatalogDetailTemplate => ("{} herramientas · {} instaladas", "{} tools · {} installed"),
        ColInstalled => ("", ""),
        ColName => ("NOMBRE", "NAME"),
        ColCategory => ("CATEGORÍA", "CATEGORY"),
        ColPackage => ("PAQUETE (DEB)", "PACKAGE (DEB)"),
        ColLicense => ("LICENCIA", "LICENSE"),
        ColStatus => ("ESTADO", "STATUS"),
        StatusInstalled => ("INSTALADA", "INSTALLED"),
        StatusMissing => ("FALTA", "MISSING"),

        ScanTitle => ("Ejecución", "Scan"),
        ScanNoToolSelected => (
            "Sin herramienta seleccionada. Abrí el Catálogo (2), elegí una con Enter y volvé acá.",
            "No tool selected. Open the Catalog (2), pick one with Enter and come back here.",
        ),
        ScanIdleTemplate => ("Listo para ejecutar {}. Pulsá N para configurar la corrida.", "Ready to run {}. Press N to set up the run."),
        ScanRunningTemplate => ("Ejecutando: {}", "Running: {}"),
        ScanFinishedOkTemplate => ("Finalizado ({}) · código {}", "Finished ({}) · exit {}"),
        ScanFinishedErrTemplate => ("Finalizado con error ({}) · código {}", "Finished with error ({}) · exit {}"),
        ScanOutputEmpty => ("(sin salida todavía)", "(no output yet)"),
        ScanWaitingOutput => ("ejecutando, esperando salida…", "running, waiting for output…"),
        ScanPickFromCatalog => ("elegí una herramienta en el Catálogo", "pick a tool in the Catalog"),
        ScanNotInstalledTemplate => (
            "{} no está instalada. Instalala desde el Catálogo (tecla I) o con tu gestor de paquetes.",
            "{} is not installed. Install it from the Catalog (key I) or with your package manager.",
        ),

        LogsEmpty => ("Todavía no ejecutaste ninguna herramienta.", "You have not run any tool yet."),
        LogsDetailTemplate => ("{} corrida(s)", "{} run(s)"),
        LogRunningTag => ("EN CURSO", "RUNNING"),
        LogOkTag => ("OK", "OK"),
        LogErrTag => ("ERROR", "ERROR"),

        FormRunTitleTemplate => ("ejecutar · {}", "run · {}"),
        FormLabelTemplate => ("PLANTILLA", "TEMPLATE"),
        FormLabelTarget => ("OBJETIVO (host/ip/url/archivo)", "TARGET (host/ip/url/file)"),
        FormLabelArgs => ("ARGUMENTOS ({} = objetivo)", "ARGUMENTS ({} = target)"),
        FormNextField => ("campo siguiente", "next field"),
        FormTemplateHint => ("plantilla", "template"),
        FormErrNoCommand => ("no hay comando para ejecutar", "there is no command to run"),
        FormDestructiveWarning => ("⚠ estos argumentos modifican o eliminan archivos del objetivo", "⚠ these arguments modify or delete files on the target"),
        FormDestructiveConfirm => ("ACCIÓN DESTRUCTIVA: pulsá S para confirmar, cualquier otra tecla para volver", "DESTRUCTIVE ACTION: press Y to confirm, any other key to go back"),

        InfoTitleTemplate => ("herramienta · {}", "tool · {}"),
        InfoHint => ("↑↓ scroll · cualquier otra tecla cierra", "↑↓ scroll · any other key closes"),
        InfoFieldCategory => ("Categoría", "Category"),
        InfoFieldLicense => ("Licencia", "License"),
        InfoFieldBinary => ("Binario", "Binary"),
        InfoFieldStatus => ("Estado", "Status"),
        InfoFieldPurpose => ("Propósito", "Purpose"),
        InfoFieldInstall => ("Instalación", "Install"),
        InfoFieldTemplates => ("Plantillas", "Templates"),
        InfoFieldGo => ("go install", "go install"),
        InfoFieldCopr => ("COPR", "COPR"),

        ConfigFormTitle => ("configuración", "settings"),
        ConfigFieldRefreshLabel => ("Refresco (s)", "Refresh (s)"),
        ConfigFieldRefreshHint => ("cada cuántos segundos se re-detectan las herramientas", "how often tools are re-detected, in seconds"),
        ConfigFieldTimeoutLabel => ("Timeout (s)", "Timeout (s)"),
        ConfigFieldTimeoutHint => ("segundos máximos por comando antes de abortarlo", "max seconds per command before aborting it"),
        ConfigFieldLanguageLabel => ("Idioma", "Language"),
        ConfigFieldLanguageHint => ("◀▶ cambia entre español e inglés", "◀▶ switch between Spanish and English"),
        ConfigErrInvalidTemplate => ("valor inválido en {}", "invalid value in {}"),
        ConfigErrZeroTemplate => ("{} no puede ser cero", "{} cannot be zero"),
        ConfigErrCreateDirTemplate => ("no se pudo crear {}", "could not create {}"),
        ConfigErrSerialize => ("no se pudo serializar la config", "could not serialize the settings"),
        ConfigErrWriteTemplate => ("no se pudo escribir {}", "could not write {}"),
        ConfigErrNoUserDir => ("no se pudo resolver el directorio de config del usuario", "could not resolve the user config directory"),

        SetupGateTitle => ("Verificación del sistema", "System check"),
        SetupPanelTitle => ("Verificación del sistema", "System check"),
        SetupStatusAllOk => ("todo listo para operar", "everything ready to operate"),
        SetupStatusActionRequiredWithPmTemplate => ("acción requerida · gestor: {}", "action required · manager: {}"),
        SetupStatusActionRequired => ("acción requerida", "action required"),
        SetupStripeAllOk => ("SISTEMA OPERATIVO", "SYSTEM READY"),
        SetupStripeGateBlocking => ("REVISAR ANTES DE CONTINUAR", "REVIEW BEFORE CONTINUING"),
        SetupStripePanelBlocking => ("HAY ELEMENTOS PENDIENTES", "THERE ARE PENDING ITEMS"),
        SetupKeyChoose => ("elegir", "choose"),
        SetupKeyRecheck => ("re-verificar", "re-check"),
        SetupKeyContinueAnyway => ("continuar igual", "continue anyway"),
        SetupKeyExit => ("salir", "exit"),

        DepPackageManagerLabel => ("Gestor de paquetes", "Package manager"),
        DepPackageManagerOkTemplate => ("detectado: {}", "detected: {}"),
        DepPackageManagerMissing => ("no se detectó ninguno (apt/dnf/pacman/...)", "none detected (apt/dnf/pacman/...)"),
        DepToolsLabel => ("Herramientas", "Tools"),
        DepToolsOkTemplate => ("{} de {} instaladas", "{} of {} installed"),
        DepToolsNoneTemplate => ("ninguna de {} instalada", "none of {} installed"),
        DepPrivilegeLabel => ("Privilegios", "Privileges"),
        DepPrivilegeRoot => ("root: captura y sockets crudos disponibles", "root: raw sockets and capture available"),
        DepPrivilegeUser => ("usuario normal: algunas herramientas pedirán sudo", "regular user: some tools will ask for sudo"),
        DepFixturesLabel => ("Modo", "Mode"),
        DepFixturesOn => ("fixtures (datos simulados, sin ejecutar binarios)", "fixtures (simulated data, no binaries run)"),
        DepFixturesOff => ("real (ejecuta los binarios instalados)", "real (runs the installed binaries)"),

        MainFixturesSimulated => ("(modo fixtures: instalación simulada)", "(fixtures mode: simulated install)"),
        MainPressEnterToReturn => ("-- ENTER para volver al panel --", "-- press ENTER to return to the panel --"),
        MainFixDone => ("Comando ejecutado correctamente.", "Command executed successfully."),
        MainFixError => ("El comando terminó con error (ver la salida arriba).", "The command finished with an error (see output above)."),
        MainFixExecErrTemplate => ("No se pudo ejecutar el comando: {}", "Could not run the command: {}"),
        MainNoInstallCommand => ("No hay comando de instalación para esta herramienta en este sistema.", "There is no install command for this tool on this system."),
        MainAltInstallTemplate => ("{}: instalando vía {}", "{}: installing via {}"),
        MainNoAltInstall => ("No hay vía alternativa (go install / COPR) para esta herramienta.", "No alternative install path (go install / COPR) for this tool."),
        MainScanStartedTemplate => ("Ejecutando {}", "Running {}"),
        MainSentToScanTemplate => ("{} seleccionada · pestaña Ejecutar (3)", "{} selected · Scan tab (3)"),
        MainRerunNeedsConfirm => ("La última corrida es destructiva: revisala y confirmá", "The last run is destructive: review it and confirm"),
        MainConfigLoadErrTemplate => ("No se pudo cargar la configuración, se usan los valores por defecto: {}", "Could not load the settings, using defaults: {}"),
        MainConfigSaveErrTemplate => ("Ajustes aplicados sólo en esta sesión, no se pudieron guardar: {}", "Settings applied for this session only, could not be saved: {}"),
        MainConfigSavedTemplate => ("Configuración guardada en {}", "Settings saved to {}"),
    }
}

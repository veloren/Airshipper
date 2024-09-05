use msi::*;
use std::{
    fs::File,
    io::{Cursor, Write},
    path::Path,
};
use uuid::Uuid;
use zip::{ZipWriter};

// run via:
//  cargo run -p airshipper --bin=msi --features="bin_msi"

fn main() -> Result<(), Box<dyn std::error::Error>> {
    /*let _ = display_file(
    include_bytes!("airshipper-windows.msi").as_ref(),
    );*/

    // read version from Cargo.toml
    let cargo_toml = include_str!("../../../Cargo.toml");
    #[derive(serde::Deserialize)]
    struct Package {
       version: String,
    }
    #[derive(serde::Deserialize)]
    struct CargoToml {
       package: Package,
    }
    let ctoml: CargoToml = toml::from_str(cargo_toml).expect("failed to parse Cargo.toml");
    let version = ctoml.package.version;
    println!("version detected {}", version);

    // loading .exe at runtime, because it might not exist during compile



    println!("Building .msi for airshipper");
    let package = build_airshipper(&version, &[]);
    // Close the package and get the cursor back out.
    let cursor = package.into_inner()?;
    let data = cursor.into_inner();
    let mut bytes = bytes::BytesMut::new();
    bytes.extend_from_slice(&data);
    let bytes = bytes.freeze();

    let mut file = File::create("airshipper-windows.msi")?;
    file.write_all(&bytes)?;
    file.flush()?;

    println!("airshipper-windows.msi written");

    Ok(())
}

// version, e.g. `0.13.0`
fn build_airshipper(version: &str, airshipper_exe: &[u8]) -> Package<Cursor<Vec<u8>>> {
    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);
    let mut package = Package::create(PackageType::Installer, cursor).unwrap();
    // Set some summary information:
    let summary = package.summary_info_mut();

    summary.set_codepage(msi::CodePage::Windows1252);
    summary.set_title("Installation Database");
    summary.set_subject("Provides automatic updates for the voxel RPG Veloren.");
    summary.set_author("veloren.net");

    let my_uuid = Uuid::try_parse("c6715dd2-38e8-4c7f-9648-559ca5ac3789").unwrap();
    summary.set_uuid(my_uuid);
    summary.set_arch("x64");
    summary.set_languages(&[msi::Language::from_tag("en-US")]);
    summary.set_creation_time_to_now();
    summary.set_creating_application("airshipper-msi builder");
    summary.set_comments(
        "This installer database contains the logic and data required to install \
         Airshipper.",
    );

    const ADMIN_EXECUTE_SEQUENCE: &str = "AdminExecuteSequence";
    const ADMIN_UI_SEQUENCE: &str = "AdminUISequence";
    const ADVT_EXECUTE_SEQUENCE: &str = "AdvtExecuteSequence";
    const APP_SEARCH: &str = "AppSearch";
    const BINARY: &str = "Binary";
    const CHECKBOX: &str = "CheckBox";
    const COMPONENT: &str = "Component";
    const CONTROL: &str = "Control";
    const CONTROL_CONDITION: &str = "ControlCondition";
    const CONTROL_EVENT: &str = "ControlEvent";
    const CREATE_FOLDER: &str = "CreateFolder";
    const CUSTOM_ACTION: &str = "CustomAction";
    const DIALOG: &str = "Dialog";
    const DIRECTORY: &str = "Directory";
    const ENVIRONMENT: &str = "Environment";
    const EVENT_MAPPING: &str = "EventMapping";
    const FEATURE: &str = "Feature";
    const FEATURE_COMPONENTS: &str = "FeatureComponents";
    const FILE: &str = "File";
    const ICON: &str = "Icon";
    const INSTALL_EXECUTE_SEQUENCE: &str = "InstallExecuteSequence";
    const INSTALL_UI_SEQUENCE: &str = "InstallUISequence";
    const LAUNCH_CONDITION: &str = "LaunchCondition";
    const LIST_BOX: &str = "ListBox";
    const MEDIA: &str = "Media";
    const PROPERTY: &str = "Property";
    const RADIO_BUTTON: &str = "RadioButton";
    const REG_LOCATOR: &str = "RegLocator";
    const REGISTRY: &str = "Registry";
    const REMOVE_FILE: &str = "RemoveFile";
    const SHORTCUT: &str = "Shortcut";
    const SIGNATURE: &str = "Signature";
    const TEXT_STYLE: &str = "TextStyle";
    const UI_TEXT: &str = "UIText";
    const UPGRADE: &str = "Upgrade";

    // Add default Tables
    package
        .create_table(ADMIN_EXECUTE_SEQUENCE, vec![
            Column::build("Action")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Condition")
                .nullable()
                .category(Category::Condition)
                .string(255),
            Column::build("Sequence")
                .nullable()
                .range(-4, 32767)
                .int16(),
        ])
        .unwrap();

    let query = Insert::into(ADMIN_EXECUTE_SEQUENCE)
        .row(vec![
            Value::from("CostInitialize"),
            Value::Null,
            Value::from(800),
        ])
        .row(vec![Value::from("FileCost"), Value::Null, Value::from(900)])
        .row(vec![
            Value::from("CostFinalize"),
            Value::Null,
            Value::from(1000),
        ])
        .row(vec![
            Value::from("InstallValidate"),
            Value::Null,
            Value::from(1400),
        ])
        .row(vec![
            Value::from("InstallInitialize"),
            Value::Null,
            Value::from(1500),
        ])
        .row(vec![
            Value::from("InstallAdminPackage"),
            Value::Null,
            Value::from(3900),
        ])
        .row(vec![
            Value::from("InstallFiles"),
            Value::Null,
            Value::from(4000),
        ])
        .row(vec![
            Value::from("InstallFinalize"),
            Value::Null,
            Value::from(6600),
        ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(ADMIN_UI_SEQUENCE, vec![
            Column::build("Action")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Condition")
                .nullable()
                .category(Category::Condition)
                .string(255),
            Column::build("Sequence")
                .nullable()
                .range(-4, 32767)
                .int16(),
        ])
        .unwrap();

    let query = Insert::into(ADMIN_UI_SEQUENCE)
        .row(vec![
            Value::from("CostInitialize"),
            Value::Null,
            Value::from(800),
        ])
        .row(vec![Value::from("FileCost"), Value::Null, Value::from(900)])
        .row(vec![
            Value::from("CostFinalize"),
            Value::Null,
            Value::from(1000),
        ])
        .row(vec![
            Value::from("FatalError"),
            Value::Null,
            Value::from(-3),
        ])
        .row(vec![Value::from("UserExit"), Value::Null, Value::from(-2)])
        .row(vec![
            Value::from("ExitDialog"),
            Value::Null,
            Value::from(-1),
        ])
        .row(vec![
            Value::from("ExecuteAction"),
            Value::Null,
            Value::from(1300),
        ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(ADVT_EXECUTE_SEQUENCE, vec![
            Column::build("Action")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Condition")
                .nullable()
                .category(Category::Condition)
                .string(255),
            Column::build("Sequence")
                .nullable()
                .range(-4, 32767)
                .int16(),
        ])
        .unwrap();

    let query = Insert::into(ADVT_EXECUTE_SEQUENCE)
        .row(vec![
            Value::from("CostInitialize"),
            Value::Null,
            Value::from(800),
        ])
        .row(vec![
            Value::from("CostFinalize"),
            Value::Null,
            Value::from(1000),
        ])
        .row(vec![
            Value::from("InstallValidate"),
            Value::Null,
            Value::from(1400),
        ])
        .row(vec![
            Value::from("InstallInitialize"),
            Value::Null,
            Value::from(1500),
        ])
        .row(vec![
            Value::from("InstallFinalize"),
            Value::Null,
            Value::from(6300),
        ])
        .row(vec![
            Value::from("CreateShortcuts"),
            Value::Null,
            Value::from(4500),
        ])
        .row(vec![
            Value::from("PublishFeatures"),
            Value::Null,
            Value::from(6300),
        ])
        .row(vec![
            Value::from("PublishProduct"),
            Value::Null,
            Value::from(6400),
        ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(APP_SEARCH, vec![
            Column::build("Property")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Signature_")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
        ])
        .unwrap();

    let query = Insert::into(APP_SEARCH).row(vec![
        Value::from("APPLICATIONFOLDER"),
        Value::from("FindInstallLocation"),
    ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(BINARY, vec![
            Column::build("Name")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Data").category(Category::Binary).binary(),
        ])
        .unwrap();
    let query = Insert::into("Binary")
        .row(vec![Value::from("WixUI_Bmp_Banner"), Value::from("Name")])
        .row(vec![Value::from("WixUI_Bmp_Dialog"), Value::from("Name")])
        .row(vec![Value::from("WixUI_Ico_Exclam"), Value::from("Name")])
        .row(vec![Value::from("WixUI_Ico_Info"), Value::from("Name")])
        .row(vec![Value::from("WixUI_Bmp_New"), Value::from("Name")])
        .row(vec![Value::from("WixUI_Bmp_Up"), Value::from("Name")])
        .row(vec![Value::from("WixCA"), Value::from("Name")]);
    package.insert_rows(query).unwrap();

    package
        .create_table(CHECKBOX, vec![
            Column::build("Property").primary_key().string(72),
            Column::build("Value").nullable().string(64),
        ])
        .unwrap();
    let query = Insert::into(CHECKBOX).row(vec![
        Value::from("WIXUI_EXITDIALOGOPTIONALCHECKBOX"),
        Value::from("1"),
    ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(COMPONENT, vec![
            Column::build("Component")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("ComponentId")
                .nullable()
                .category(Category::Guid)
                .string(38),
            Column::build("Directory_")
                .category(Category::Identifier)
                .string(72),
            Column::build("Attributes").int16(),
            Column::build("Condition")
                .category(Category::Condition)
                .nullable()
                .string(255),
            Column::build("KeyPath")
                .category(Category::Identifier)
                .nullable()
                .string(72),
        ])
        .unwrap();
    let query = Insert::into(COMPONENT)
        .row(vec![
            Value::from("ApplicationShortcut"),
            Value::from("{E2CD5A52-9B7C-410F-8C85-0D834C435A43}"),
            Value::from("ApplicationProgramsFolder"),
            Value::from(260),
            Value::Null,
            Value::from("reg973689048B04F03AA9C98C5A1F6AE6D3"),
        ])
        .row(vec![
            Value::from("airshipper.exe"),
            Value::from("{DD1E4A01-FF62-4D96-A9CE-A5D7D7548FAD}"),
            Value::from("APPLICATIONFOLDER"),
            Value::from(256),
            Value::Null,
            Value::from("airshipper.exe"),
        ])
        .row(vec![
            Value::from("Path"),
            Value::from("{4A62682D-9DFD-4203-AD35-1E4300AD6A5A}"),
            Value::from("APPLICATIONFOLDER"),
            Value::from(256),
            Value::Null,
            Value::Null,
        ])
        .row(vec![
            Value::from("ApplicationShortcutDesktop"),
            Value::from("{C2601E65-EDF8-4F5A-94FF-0B3E800FE375}"),
            Value::from("DesktopFolder"),
            Value::from(4),
            Value::Null,
            Value::from("reg65FB3BC78BA56162E6CFE88513453D5D"),
        ])
        .row(vec![
            Value::from("ApplicationShortcutCompatibilityDesktop"),
            Value::from("{7B3E27BA-0FD5-4DFB-9BE3-9614EC25FD9A}"),
            Value::from("DesktopFolder"),
            Value::from(4),
            Value::Null,
            Value::from("regB8E49E39311754CE2F424A110DF17A1B"),
        ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(CONTROL, vec![
            Column::build("Dialog_")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Control")
                .primary_key()
                .category(Category::Identifier)
                .string(50),
            Column::build("Type")
                .category(Category::Identifier)
                .string(20),
            Column::build("X").range(0, 32767).int16(),
            Column::build("Y").range(0, 32767).int16(),
            Column::build("Width").range(0, 32767).int16(),
            Column::build("Height").range(0, 32767).int16(),
            Column::build("Attributes")
                .nullable()
                .range(0, 2147483647)
                .int32(),
            Column::build("Property")
                .nullable()
                .category(Category::Identifier)
                .string(72),
            Column::build("Text")
                .nullable()
                .localizable()
                .category(Category::Formatted)
                .string(0),
            Column::build("Control_Next")
                .nullable()
                .category(Category::Identifier)
                .string(50),
            Column::build("Help")
                .nullable()
                .localizable()
                .category(Category::Text)
                .string(50),
        ])
        .unwrap();

    let query = Insert::into(CONTROL).row(vec![Value::from(r#"FatalError"#), Value::from(r#"Description"#), Value::from(r#"Text"#), Value::from(135), Value::from(70), Value::from(220), Value::from(80), Value::from(196611), Value::Null, Value::from(r#"[ProductName] Setup Wizard ended prematurely because of an error. Your system has not been modified. To install this program at a later time, run Setup Wizard again. Click the Finish button to exit the Setup Wizard."#), Value::Null, Value::Null]).row(vec![Value::from(r#"FatalError"#), Value::from(r#"Title"#), Value::from(r#"Text"#), Value::from(135), Value::from(20), Value::from(220), Value::from(60), Value::from(196611), Value::Null, Value::from(r#"{\WixUI_Font_Bigger}[ProductName] Setup Wizard ended prematurely"#), Value::Null, Value::Null]).row(vec![Value::from(r#"FatalError"#), Value::from("Cancel"), Value::from(r#"PushButton"#), Value::from(304), Value::from(243), Value::from(56), Value::from(17), Value::from(1), Value::Null, Value::from("Cancel"), Value::from(r#"Bitmap"#), Value::Null]).row(vec![Value::from(r#"FatalError"#), Value::from("Finish"), Value::from(r#"PushButton"#), Value::from(236), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&Finish"#), Value::from("Cancel"), Value::Null]).row(vec![Value::from(r#"FatalError"#), Value::from(r#"Bitmap"#), Value::from(r#"Bitmap"#), Value::from(0), Value::from(0), Value::from(370), Value::from(234), Value::from(1), Value::Null, Value::from(r#"WixUI_Bmp_Dialog"#), Value::from(r#"Back"#), Value::Null]).row(vec![Value::from(r#"FatalError"#), Value::from(r#"Back"#), Value::from(r#"PushButton"#), Value::from(180), Value::from(243), Value::from(56), Value::from(17), Value::from(1), Value::Null, Value::from(r#"&Back"#), Value::from("Finish"), Value::Null]).row(vec![Value::from(r#"FatalError"#), Value::from(r#"BottomLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(234), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"UserExit"#), Value::from(r#"Description"#), Value::from(r#"Text"#), Value::from(135), Value::from(80), Value::from(220), Value::from(80), Value::from(196611), Value::Null, Value::from(r#"[ProductName] setup was interrupted. Your system has not been modified. To install this program at a later time, please run the installation again. Click the Finish button to exit the Setup Wizard."#), Value::Null, Value::Null]).row(vec![Value::from(r#"UserExit"#), Value::from(r#"Title"#), Value::from(r#"Text"#), Value::from(135), Value::from(20), Value::from(220), Value::from(60), Value::from(196611), Value::Null, Value::from(r#"{\WixUI_Font_Bigger}[ProductName] Setup Wizard was interrupted"#), Value::Null, Value::Null]).row(vec![Value::from(r#"UserExit"#), Value::from("Cancel"), Value::from(r#"PushButton"#), Value::from(304), Value::from(243), Value::from(56), Value::from(17), Value::from(1), Value::Null, Value::from("Cancel"), Value::from(r#"Bitmap"#), Value::Null]).row(vec![Value::from(r#"UserExit"#), Value::from("Finish"), Value::from(r#"PushButton"#), Value::from(236), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&Finish"#), Value::from("Cancel"), Value::Null]).row(vec![Value::from(r#"UserExit"#), Value::from(r#"Bitmap"#), Value::from(r#"Bitmap"#), Value::from(0), Value::from(0), Value::from(370), Value::from(234), Value::from(1), Value::Null, Value::from(r#"WixUI_Bmp_Dialog"#), Value::from(r#"Back"#), Value::Null]).row(vec![Value::from(r#"UserExit"#), Value::from(r#"Back"#), Value::from(r#"PushButton"#), Value::from(180), Value::from(243), Value::from(56), Value::from(17), Value::from(1), Value::Null, Value::from(r#"&Back"#), Value::from("Finish"), Value::Null]).row(vec![Value::from(r#"UserExit"#), Value::from(r#"BottomLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(234), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"ExitDialog"#), Value::from(r#"Description"#), Value::from(r#"Text"#), Value::from(135), Value::from(70), Value::from(220), Value::from(40), Value::from(196611), Value::Null, Value::from(r#"Click the Finish button to exit the Setup Wizard."#), Value::Null, Value::Null]).row(vec![Value::from(r#"ExitDialog"#), Value::from(r#"Title"#), Value::from(r#"Text"#), Value::from(135), Value::from(20), Value::from(220), Value::from(60), Value::from(196611), Value::Null, Value::from(r#"{\WixUI_Font_Bigger}Completed the [ProductName] Setup Wizard"#), Value::Null, Value::Null]).row(vec![Value::from(r#"ExitDialog"#), Value::from("Cancel"), Value::from(r#"PushButton"#), Value::from(304), Value::from(243), Value::from(56), Value::from(17), Value::from(1), Value::Null, Value::from("Cancel"), Value::from(r#"Bitmap"#), Value::Null]).row(vec![Value::from(r#"ExitDialog"#), Value::from("Finish"), Value::from(r#"PushButton"#), Value::from(236), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&Finish"#), Value::from("Cancel"), Value::Null]).row(vec![Value::from(r#"ExitDialog"#), Value::from(r#"Bitmap"#), Value::from(r#"Bitmap"#), Value::from(0), Value::from(0), Value::from(370), Value::from(234), Value::from(1), Value::Null, Value::from(r#"WixUI_Bmp_Dialog"#), Value::from(r#"Back"#), Value::Null]).row(vec![Value::from(r#"ExitDialog"#), Value::from(r#"Back"#), Value::from(r#"PushButton"#), Value::from(180), Value::from(243), Value::from(56), Value::from(17), Value::from(1), Value::Null, Value::from(r#"&Back"#), Value::from(r#"OptionalCheckBox"#), Value::Null]).row(vec![Value::from(r#"ExitDialog"#), Value::from(r#"BottomLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(234), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"ExitDialog"#), Value::from(r#"OptionalCheckBox"#), Value::from(r#"CheckBox"#), Value::from(135), Value::from(190), Value::from(220), Value::from(40), Value::from(2), Value::from(r#"WIXUI_EXITDIALOGOPTIONALCHECKBOX"#), Value::from(r#"[WIXUI_EXITDIALOGOPTIONALCHECKBOXTEXT]"#), Value::from("Finish"), Value::Null]).row(vec![Value::from(r#"ExitDialog"#), Value::from(r#"OptionalText"#), Value::from(r#"Text"#), Value::from(135), Value::from(110), Value::from(220), Value::from(80), Value::from(196610), Value::Null, Value::from(r#"[WIXUI_EXITDIALOGOPTIONALTEXT]"#), Value::Null, Value::Null]).row(vec![Value::from(r#"ErrorDlg"#), Value::from(r#"N"#), Value::from(r#"PushButton"#), Value::from(100), Value::from(80), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&No"#), Value::Null, Value::Null]).row(vec![Value::from(r#"ErrorDlg"#), Value::from(r#"Y"#), Value::from(r#"PushButton"#), Value::from(100), Value::from(80), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&Yes"#), Value::Null, Value::Null]).row(vec![Value::from(r#"ErrorDlg"#), Value::from(r#"ErrorText"#), Value::from(r#"Text"#), Value::from(48), Value::from(15), Value::from(205), Value::from(60), Value::from(131075), Value::Null, Value::from(r#"Information text"#), Value::Null, Value::Null]).row(vec![Value::from(r#"ErrorDlg"#), Value::from(r#"A"#), Value::from(r#"PushButton"#), Value::from(100), Value::from(80), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from("Cancel"), Value::Null, Value::Null]).row(vec![Value::from(r#"ErrorDlg"#), Value::from(r#"C"#), Value::from(r#"PushButton"#), Value::from(100), Value::from(80), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from("Cancel"), Value::Null, Value::Null]).row(vec![Value::from(r#"ErrorDlg"#), Value::from(r#"ErrorIcon"#), Value::from(r#"Icon"#), Value::from(15), Value::from(15), Value::from(24), Value::from(24), Value::from(5242881), Value::Null, Value::from(r#"WixUI_Ico_Info"#), Value::Null, Value::from(r#"Information icon|"#)]).row(vec![Value::from(r#"ErrorDlg"#), Value::from(r#"I"#), Value::from(r#"PushButton"#), Value::from(100), Value::from(80), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&Ignore"#), Value::Null, Value::Null]).row(vec![Value::from(r#"ErrorDlg"#), Value::from(r#"O"#), Value::from(r#"PushButton"#), Value::from(100), Value::from(80), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from("OK"), Value::Null, Value::Null]).row(vec![Value::from(r#"ErrorDlg"#), Value::from(r#"R"#), Value::from(r#"PushButton"#), Value::from(100), Value::from(80), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&Retry"#), Value::Null, Value::Null]).row(vec![Value::from(r#"FilesInUse"#), Value::from(r#"Description"#), Value::from(r#"Text"#), Value::from(20), Value::from(23), Value::from(280), Value::from(20), Value::from(196611), Value::Null, Value::from(r#"Some files that need to be updated are currently in use."#), Value::Null, Value::Null]).row(vec![Value::from(r#"FilesInUse"#), Value::from(r#"Text"#), Value::from(r#"Text"#), Value::from(20), Value::from(55), Value::from(330), Value::from(30), Value::from(3), Value::Null, Value::from(r#"The following applications are using files that need to be updated by this setup. Close these applications and then click &Retry to continue setup or Exit to exit it."#), Value::Null, Value::Null]).row(vec![Value::from(r#"FilesInUse"#), Value::from(r#"Title"#), Value::from(r#"Text"#), Value::from(15), Value::from(6), Value::from(200), Value::from(15), Value::from(196611), Value::Null, Value::from(r#"{\WixUI_Font_Title}Files in Use"#), Value::Null, Value::Null]).row(vec![Value::from(r#"FilesInUse"#), Value::from(r#"BottomLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(234), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"FilesInUse"#), Value::from("Retry"), Value::from(r#"PushButton"#), Value::from(304), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&Retry"#), Value::from(r#"Ignore"#), Value::Null]).row(vec![Value::from(r#"FilesInUse"#), Value::from(r#"Ignore"#), Value::from(r#"PushButton"#), Value::from(235), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&Ignore"#), Value::from(r#"Exit"#), Value::Null]).row(vec![Value::from(r#"FilesInUse"#), Value::from(r#"Exit"#), Value::from(r#"PushButton"#), Value::from(166), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"E&xit"#), Value::from(r#"BannerBitmap"#), Value::Null]).row(vec![Value::from(r#"FilesInUse"#), Value::from(r#"BannerBitmap"#), Value::from(r#"Bitmap"#), Value::from(0), Value::from(0), Value::from(370), Value::from(44), Value::from(1), Value::Null, Value::from(r#"WixUI_Bmp_Banner"#), Value::from("Retry"), Value::Null]).row(vec![Value::from(r#"FilesInUse"#), Value::from(r#"BannerLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(44), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"FilesInUse"#), Value::from(r#"List"#), Value::from(r#"ListBox"#), Value::from(20), Value::from(87), Value::from(330), Value::from(130), Value::from(7), Value::from(r#"FileInUseProcess"#), Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"MsiRMFilesInUse"#), Value::from(r#"Description"#), Value::from(r#"Text"#), Value::from(20), Value::from(23), Value::from(280), Value::from(20), Value::from(196611), Value::Null, Value::from(r#"Some files that need to be updated are currently in use."#), Value::Null, Value::Null]).row(vec![Value::from(r#"MsiRMFilesInUse"#), Value::from(r#"Text"#), Value::from(r#"Text"#), Value::from(20), Value::from(55), Value::from(330), Value::from(45), Value::from(3), Value::Null, Value::from(r#"The following applications are using files that need to be updated by this setup. You can let Setup Wizard close them and attempt to restart them or reboot the machine later."#), Value::Null, Value::Null]).row(vec![Value::from(r#"MsiRMFilesInUse"#), Value::from(r#"Title"#), Value::from(r#"Text"#), Value::from(15), Value::from(6), Value::from(200), Value::from(15), Value::from(196611), Value::Null, Value::from(r#"{\WixUI_Font_Title}Files in Use"#), Value::Null, Value::Null]).row(vec![Value::from(r#"MsiRMFilesInUse"#), Value::from("Cancel"), Value::from(r#"PushButton"#), Value::from(304), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from("Cancel"), Value::from(r#"ShutdownOption"#), Value::Null]).row(vec![Value::from(r#"MsiRMFilesInUse"#), Value::from("OK"), Value::from(r#"PushButton"#), Value::from(240), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from("OK"), Value::from("Cancel"), Value::Null]).row(vec![Value::from(r#"MsiRMFilesInUse"#), Value::from(r#"BottomLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(234), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"MsiRMFilesInUse"#), Value::from(r#"BannerBitmap"#), Value::from(r#"Bitmap"#), Value::from(0), Value::from(0), Value::from(370), Value::from(44), Value::from(1), Value::Null, Value::from(r#"WixUI_Bmp_Banner"#), Value::from("OK"), Value::Null]).row(vec![Value::from(r#"MsiRMFilesInUse"#), Value::from(r#"BannerLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(44), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"MsiRMFilesInUse"#), Value::from(r#"List"#), Value::from(r#"ListBox"#), Value::from(20), Value::from(100), Value::from(330), Value::from(80), Value::from(7), Value::from(r#"FileInUseProcess"#), Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"MsiRMFilesInUse"#), Value::from(r#"ShutdownOption"#), Value::from(r#"RadioButtonGroup"#), Value::from(26), Value::from(190), Value::from(305), Value::from(45), Value::from(3), Value::from(r#"WixUIRMOption"#), Value::Null, Value::from(r#"BannerBitmap"#), Value::Null]).row(vec![Value::from(r#"PrepareDlg"#), Value::from(r#"Description"#), Value::from(r#"Text"#), Value::from(135), Value::from(70), Value::from(220), Value::from(20), Value::from(196611), Value::Null, Value::from(r#"Please wait while the Setup Wizard prepares to guide you through the installation."#), Value::Null, Value::Null]).row(vec![Value::from(r#"PrepareDlg"#), Value::from(r#"Title"#), Value::from(r#"Text"#), Value::from(135), Value::from(20), Value::from(220), Value::from(60), Value::from(196611), Value::Null, Value::from(r#"{\WixUI_Font_Bigger}Welcome to the [ProductName] Setup Wizard"#), Value::Null, Value::Null]).row(vec![Value::from(r#"PrepareDlg"#), Value::from("Cancel"), Value::from(r#"PushButton"#), Value::from(304), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from("Cancel"), Value::from(r#"Bitmap"#), Value::Null]).row(vec![Value::from(r#"PrepareDlg"#), Value::from(r#"Bitmap"#), Value::from(r#"Bitmap"#), Value::from(0), Value::from(0), Value::from(370), Value::from(234), Value::from(1), Value::Null, Value::from(r#"WixUI_Bmp_Dialog"#), Value::from("Cancel"), Value::Null]).row(vec![Value::from(r#"PrepareDlg"#), Value::from(r#"Back"#), Value::from(r#"PushButton"#), Value::from(180), Value::from(243), Value::from(56), Value::from(17), Value::from(1), Value::Null, Value::from(r#"&Back"#), Value::Null, Value::Null]).row(vec![Value::from(r#"PrepareDlg"#), Value::from(r#"BottomLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(234), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"PrepareDlg"#), Value::from("Next"), Value::from(r#"PushButton"#), Value::from(236), Value::from(243), Value::from(56), Value::from(17), Value::from(1), Value::Null, Value::from(r#"&Next"#), Value::Null, Value::Null]).row(vec![Value::from(r#"PrepareDlg"#), Value::from(r#"ActionData"#), Value::from(r#"Text"#), Value::from(135), Value::from(125), Value::from(220), Value::from(30), Value::from(196611), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"PrepareDlg"#), Value::from(r#"ActionText"#), Value::from(r#"Text"#), Value::from(135), Value::from(100), Value::from(220), Value::from(20), Value::from(196611), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"CancelDlg"#), Value::from(r#"Text"#), Value::from(r#"Text"#), Value::from(48), Value::from(15), Value::from(194), Value::from(30), Value::from(131075), Value::Null, Value::from(r#"Are you sure you want to cancel [ProductName] installation?"#), Value::Null, Value::Null]).row(vec![Value::from(r#"CancelDlg"#), Value::from(r#"Icon"#), Value::from(r#"Icon"#), Value::from(15), Value::from(15), Value::from(24), Value::from(24), Value::from(5242881), Value::Null, Value::from(r#"WixUI_Ico_Info"#), Value::Null, Value::from(r#"Information icon|"#)]).row(vec![Value::from(r#"CancelDlg"#), Value::from("No"), Value::from(r#"PushButton"#), Value::from(132), Value::from(57), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&No"#), Value::from(r#"Yes"#), Value::Null]).row(vec![Value::from(r#"CancelDlg"#), Value::from(r#"Yes"#), Value::from(r#"PushButton"#), Value::from(72), Value::from(57), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&Yes"#), Value::from("No"), Value::Null]).row(vec![Value::from(r#"ProgressDlg"#), Value::from("Cancel"), Value::from(r#"PushButton"#), Value::from(304), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from("Cancel"), Value::from(r#"BannerBitmap"#), Value::Null]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"Back"#), Value::from(r#"PushButton"#), Value::from(180), Value::from(243), Value::from(56), Value::from(17), Value::from(1), Value::Null, Value::from(r#"&Back"#), Value::from("Next"), Value::Null]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"BottomLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(234), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"BannerBitmap"#), Value::from(r#"Bitmap"#), Value::from(0), Value::from(0), Value::from(370), Value::from(44), Value::from(1), Value::Null, Value::from(r#"WixUI_Bmp_Banner"#), Value::from(r#"Back"#), Value::Null]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"BannerLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(44), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"ProgressDlg"#), Value::from("Next"), Value::from(r#"PushButton"#), Value::from(236), Value::from(243), Value::from(56), Value::from(17), Value::from(1), Value::Null, Value::from(r#"&Next"#), Value::from("Cancel"), Value::Null]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"ActionText"#), Value::from(r#"Text"#), Value::from(70), Value::from(100), Value::from(285), Value::from(10), Value::from(3), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TextInstalling"#), Value::from(r#"Text"#), Value::from(20), Value::from(65), Value::from(330), Value::from(35), Value::from(131074), Value::Null, Value::from(r#"Please wait while the Setup Wizard installs [ProductName]."#), Value::Null, Value::Null]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TitleInstalling"#), Value::from(r#"Text"#), Value::from(20), Value::from(15), Value::from(330), Value::from(15), Value::from(196610), Value::Null, Value::from(r#"{\WixUI_Font_Title}Installing [ProductName]"#), Value::Null, Value::Null]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TextChanging"#), Value::from(r#"Text"#), Value::from(20), Value::from(65), Value::from(330), Value::from(35), Value::from(131074), Value::Null, Value::from(r#"Please wait while the Setup Wizard changes [ProductName]."#), Value::Null, Value::Null]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TitleChanging"#), Value::from(r#"Text"#), Value::from(20), Value::from(15), Value::from(330), Value::from(15), Value::from(196610), Value::Null, Value::from(r#"{\WixUI_Font_Title}Changing [ProductName]"#), Value::Null, Value::Null]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TextRepairing"#), Value::from(r#"Text"#), Value::from(20), Value::from(65), Value::from(330), Value::from(35), Value::from(131074), Value::Null, Value::from(r#"Please wait while the Setup Wizard repairs [ProductName]."#), Value::Null, Value::Null]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TitleRepairing"#), Value::from(r#"Text"#), Value::from(20), Value::from(15), Value::from(330), Value::from(15), Value::from(196610), Value::Null, Value::from(r#"{\WixUI_Font_Title}Repairing [ProductName]"#), Value::Null, Value::Null]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TextRemoving"#), Value::from(r#"Text"#), Value::from(20), Value::from(65), Value::from(330), Value::from(35), Value::from(131074), Value::Null, Value::from(r#"Please wait while the Setup Wizard removes [ProductName]."#), Value::Null, Value::Null]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TitleRemoving"#), Value::from(r#"Text"#), Value::from(20), Value::from(15), Value::from(330), Value::from(15), Value::from(196610), Value::Null, Value::from(r#"{\WixUI_Font_Title}Removing [ProductName]"#), Value::Null, Value::Null]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TextUpdating"#), Value::from(r#"Text"#), Value::from(20), Value::from(65), Value::from(330), Value::from(35), Value::from(131074), Value::Null, Value::from(r#"Please wait while the Setup Wizard updates [ProductName]."#), Value::Null, Value::Null]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TitleUpdating"#), Value::from(r#"Text"#), Value::from(20), Value::from(15), Value::from(330), Value::from(15), Value::from(196610), Value::Null, Value::from(r#"{\WixUI_Font_Title}Updating [ProductName]"#), Value::Null, Value::Null]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"ProgressBar"#), Value::from(r#"ProgressBar"#), Value::from(20), Value::from(115), Value::from(330), Value::from(10), Value::from(65537), Value::Null, Value::from(r#"Progress done"#), Value::Null, Value::Null]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"StatusLabel"#), Value::from(r#"Text"#), Value::from(20), Value::from(100), Value::from(50), Value::from(10), Value::from(3), Value::Null, Value::from(r#"Status:"#), Value::Null, Value::Null]).row(vec![Value::from(r#"ResumeDlg"#), Value::from(r#"Description"#), Value::from(r#"Text"#), Value::from(135), Value::from(80), Value::from(220), Value::from(60), Value::from(196611), Value::Null, Value::from(r#"The Setup Wizard will complete the installation of [ProductName] on your computer. Click Install to continue or Cancel to exit the Setup Wizard."#), Value::Null, Value::Null]).row(vec![Value::from(r#"ResumeDlg"#), Value::from(r#"Title"#), Value::from(r#"Text"#), Value::from(135), Value::from(20), Value::from(220), Value::from(60), Value::from(196611), Value::Null, Value::from(r#"{\WixUI_Font_Bigger}Resuming the [ProductName] Setup Wizard"#), Value::Null, Value::Null]).row(vec![Value::from(r#"ResumeDlg"#), Value::from("Cancel"), Value::from(r#"PushButton"#), Value::from(304), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from("Cancel"), Value::from(r#"Bitmap"#), Value::Null]).row(vec![Value::from(r#"ResumeDlg"#), Value::from(r#"Bitmap"#), Value::from(r#"Bitmap"#), Value::from(0), Value::from(0), Value::from(370), Value::from(234), Value::from(1), Value::Null, Value::from(r#"WixUI_Bmp_Dialog"#), Value::from(r#"Back"#), Value::Null]).row(vec![Value::from(r#"ResumeDlg"#), Value::from(r#"Back"#), Value::from(r#"PushButton"#), Value::from(156), Value::from(243), Value::from(56), Value::from(17), Value::from(1), Value::Null, Value::from(r#"&Back"#), Value::from("Install"), Value::Null]).row(vec![Value::from(r#"ResumeDlg"#), Value::from(r#"BottomLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(234), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"ResumeDlg"#), Value::from("Install"), Value::from(r#"PushButton"#), Value::from(212), Value::from(243), Value::from(80), Value::from(17), Value::from(8388610), Value::Null, Value::from(r#"&Install"#), Value::from("InstallNoShield"), Value::Null]).row(vec![Value::from(r#"ResumeDlg"#), Value::from("InstallNoShield"), Value::from(r#"PushButton"#), Value::from(212), Value::from(243), Value::from(80), Value::from(17), Value::from(2), Value::Null, Value::from(r#"&Install"#), Value::from("Cancel"), Value::Null]).row(vec![Value::from(r#"WaitForCostingDlg"#), Value::from(r#"Text"#), Value::from(r#"Text"#), Value::from(48), Value::from(15), Value::from(194), Value::from(30), Value::from(3), Value::Null, Value::from(r#"Please wait while the installer finishes determining your disk space requirements."#), Value::Null, Value::Null]).row(vec![Value::from(r#"WaitForCostingDlg"#), Value::from(r#"Icon"#), Value::from(r#"Icon"#), Value::from(15), Value::from(15), Value::from(24), Value::from(24), Value::from(5242881), Value::Null, Value::from(r#"WixUI_Ico_Exclam"#), Value::Null, Value::from(r#"Exclamation icon|"#)]).row(vec![Value::from(r#"WaitForCostingDlg"#), Value::from("Return"), Value::from(r#"PushButton"#), Value::from(102), Value::from(57), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&Return"#), Value::Null, Value::Null]).row(vec![Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"Description"#), Value::from(r#"Text"#), Value::from(20), Value::from(20), Value::from(280), Value::from(20), Value::from(196611), Value::Null, Value::from(r#"Disk space required for the installation exceeds available disk space."#), Value::Null, Value::Null]).row(vec![Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"Text"#), Value::from(r#"Text"#), Value::from(20), Value::from(53), Value::from(330), Value::from(90), Value::from(3), Value::Null, Value::from(r#"The highlighted volumes do not have enough disk space available for the currently selected features. You can remove some files from the highlighted volumes, install fewer features, or select a different destination drive. Alternatively, you may choose to disable the installer's rollback functionality. Disabling rollback prevents the installer from restoring your computer's original state should the installation be interrupted in any way. Click Yes if you wish to take the risk of disabling rollback."#), Value::Null, Value::Null]).row(vec![Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"Title"#), Value::from(r#"Text"#), Value::from(15), Value::from(6), Value::from(200), Value::from(15), Value::from(196611), Value::Null, Value::from(r#"{\WixUI_Font_Title}Out of Disk Space"#), Value::Null, Value::Null]).row(vec![Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"BottomLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(234), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"BannerBitmap"#), Value::from(r#"Bitmap"#), Value::from(0), Value::from(0), Value::from(370), Value::from(44), Value::from(1), Value::Null, Value::from(r#"WixUI_Bmp_Banner"#), Value::from("No"), Value::Null]).row(vec![Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"BannerLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(44), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"OutOfRbDiskDlg"#), Value::from("No"), Value::from(r#"PushButton"#), Value::from(304), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&No"#), Value::from(r#"Yes"#), Value::Null]).row(vec![Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"Yes"#), Value::from(r#"PushButton"#), Value::from(240), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&Yes"#), Value::from(r#"BannerBitmap"#), Value::Null]).row(vec![Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"VolumeList"#), Value::from(r#"VolumeCostList"#), Value::from(20), Value::from(150), Value::from(330), Value::from(70), Value::from(4587527), Value::Null, Value::from(r#"{120}{70}{70}{70}{70}"#), Value::Null, Value::Null]).row(vec![Value::from(r#"OutOfDiskDlg"#), Value::from(r#"Description"#), Value::from(r#"Text"#), Value::from(20), Value::from(20), Value::from(280), Value::from(20), Value::from(196611), Value::Null, Value::from(r#"Disk space required for the installation exceeds available disk space."#), Value::Null, Value::Null]).row(vec![Value::from(r#"OutOfDiskDlg"#), Value::from(r#"Text"#), Value::from(r#"Text"#), Value::from(20), Value::from(53), Value::from(330), Value::from(60), Value::from(3), Value::Null, Value::from(r#"The highlighted volumes do not have enough disk space available for the currently selected features. You can remove some files from the highlighted volumes, install fewer features, or select a different destination drive."#), Value::Null, Value::Null]).row(vec![Value::from(r#"OutOfDiskDlg"#), Value::from(r#"Title"#), Value::from(r#"Text"#), Value::from(15), Value::from(6), Value::from(200), Value::from(15), Value::from(196611), Value::Null, Value::from(r#"{\WixUI_Font_Title}Out of Disk Space"#), Value::Null, Value::Null]).row(vec![Value::from(r#"OutOfDiskDlg"#), Value::from("OK"), Value::from(r#"PushButton"#), Value::from(304), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from("OK"), Value::from(r#"BannerBitmap"#), Value::Null]).row(vec![Value::from(r#"OutOfDiskDlg"#), Value::from(r#"BottomLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(234), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"OutOfDiskDlg"#), Value::from(r#"BannerBitmap"#), Value::from(r#"Bitmap"#), Value::from(0), Value::from(0), Value::from(370), Value::from(44), Value::from(1), Value::Null, Value::from(r#"WixUI_Bmp_Banner"#), Value::from("OK"), Value::Null]).row(vec![Value::from(r#"OutOfDiskDlg"#), Value::from(r#"BannerLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(44), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"OutOfDiskDlg"#), Value::from(r#"VolumeList"#), Value::from(r#"VolumeCostList"#), Value::from(20), Value::from(120), Value::from(330), Value::from(100), Value::from(393223), Value::Null, Value::from(r#"{120}{70}{70}{70}{70}"#), Value::Null, Value::Null]).row(vec![Value::from(r#"WelcomeDlg"#), Value::from(r#"Description"#), Value::from(r#"Text"#), Value::from(135), Value::from(80), Value::from(220), Value::from(60), Value::from(196611), Value::Null, Value::from(r#"The Setup Wizard will install [ProductName] on your computer. Click Next to continue or Cancel to exit the Setup Wizard."#), Value::Null, Value::Null]).row(vec![Value::from(r#"WelcomeDlg"#), Value::from(r#"Title"#), Value::from(r#"Text"#), Value::from(135), Value::from(20), Value::from(220), Value::from(60), Value::from(196611), Value::Null, Value::from(r#"{\WixUI_Font_Bigger}Welcome to the [ProductName] Setup Wizard"#), Value::Null, Value::Null]).row(vec![Value::from(r#"WelcomeDlg"#), Value::from("Cancel"), Value::from(r#"PushButton"#), Value::from(304), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from("Cancel"), Value::from(r#"Bitmap"#), Value::Null]).row(vec![Value::from(r#"WelcomeDlg"#), Value::from(r#"Bitmap"#), Value::from(r#"Bitmap"#), Value::from(0), Value::from(0), Value::from(370), Value::from(234), Value::from(1), Value::Null, Value::from(r#"WixUI_Bmp_Dialog"#), Value::from(r#"Back"#), Value::Null]).row(vec![Value::from(r#"WelcomeDlg"#), Value::from(r#"Back"#), Value::from(r#"PushButton"#), Value::from(180), Value::from(243), Value::from(56), Value::from(17), Value::from(1), Value::Null, Value::from(r#"&Back"#), Value::from("Next"), Value::Null]).row(vec![Value::from(r#"WelcomeDlg"#), Value::from(r#"BottomLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(234), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"WelcomeDlg"#), Value::from("Next"), Value::from(r#"PushButton"#), Value::from(236), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&Next"#), Value::from("Cancel"), Value::Null]).row(vec![Value::from(r#"WelcomeDlg"#), Value::from(r#"PatchDescription"#), Value::from(r#"Text"#), Value::from(135), Value::from(80), Value::from(220), Value::from(60), Value::from(196611), Value::Null, Value::from(r#"The Setup Wizard will update [ProductName] on your computer. Click Next to continue or Cancel to exit the Setup Wizard."#), Value::Null, Value::Null]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"Description"#), Value::from(r#"Text"#), Value::from(25), Value::from(23), Value::from(280), Value::from(15), Value::from(196611), Value::Null, Value::from(r#"Select the way you want features to be installed."#), Value::Null, Value::Null]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"Text"#), Value::from(r#"Text"#), Value::from(25), Value::from(55), Value::from(320), Value::from(20), Value::from(3), Value::Null, Value::from(r#"Click the icons in the tree below to change the way features will be installed."#), Value::Null, Value::Null]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"Title"#), Value::from(r#"Text"#), Value::from(15), Value::from(6), Value::from(210), Value::from(15), Value::from(196611), Value::Null, Value::from(r#"{\WixUI_Font_Title}Custom Setup"#), Value::Null, Value::Null]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from("Cancel"), Value::from(r#"PushButton"#), Value::from(304), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from("Cancel"), Value::from(r#"BannerBitmap"#), Value::Null]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"Back"#), Value::from(r#"PushButton"#), Value::from(192), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&Back"#), Value::from("Next"), Value::Null]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"BottomLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(234), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"BannerBitmap"#), Value::from(r#"Bitmap"#), Value::from(0), Value::from(0), Value::from(370), Value::from(44), Value::from(1), Value::Null, Value::from(r#"WixUI_Bmp_Banner"#), Value::from("Tree"), Value::Null]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"BannerLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(44), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from("Next"), Value::from(r#"PushButton"#), Value::from(248), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&Next"#), Value::from("Cancel"), Value::Null]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from("Tree"), Value::from(r#"SelectionTree"#), Value::from(25), Value::from(85), Value::from(175), Value::from(115), Value::from(7), Value::from(r#"_BrowseProperty"#), Value::from(r#"Tree of selections"#), Value::from(r#"Browse"#), Value::Null]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"Browse"#), Value::from(r#"PushButton"#), Value::from(294), Value::from(210), Value::from(66), Value::from(17), Value::from(3), Value::Null, Value::from(r#"B&rowse..."#), Value::from(r#"Reset"#), Value::Null]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"Reset"#), Value::from(r#"PushButton"#), Value::from(10), Value::from(243), Value::from(81), Value::from(17), Value::from(3), Value::Null, Value::from(r#"Re&set"#), Value::from(r#"DiskCost"#), Value::Null]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"DiskCost"#), Value::from(r#"PushButton"#), Value::from(91), Value::from(243), Value::from(100), Value::from(17), Value::from(3), Value::Null, Value::from(r#"Disk &Usage"#), Value::from(r#"Back"#), Value::Null]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"Box"#), Value::from(r#"GroupBox"#), Value::from(210), Value::from(81), Value::from(150), Value::from(118), Value::from(3), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"ItemDescription"#), Value::from(r#"Text"#), Value::from(215), Value::from(90), Value::from(131), Value::from(50), Value::from(3), Value::Null, Value::from(r#"CustomizeDlgItemDescription-CustomizeDlgItemDescription"#), Value::Null, Value::Null]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"ItemSize"#), Value::from(r#"Text"#), Value::from(215), Value::from(140), Value::from(131), Value::from(50), Value::from(3), Value::Null, Value::from(r#"CustomizeDlgItemSize-CustomizeDlgItemSize"#), Value::Null, Value::Null]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"Location"#), Value::from(r#"Text"#), Value::from(90), Value::from(210), Value::from(200), Value::from(20), Value::from(3), Value::Null, Value::from(r#"CustomizeDlgLocation-CustomizeDlgLocation"#), Value::Null, Value::Null]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"LocationLabel"#), Value::from(r#"Text"#), Value::from(25), Value::from(210), Value::from(65), Value::from(10), Value::from(3), Value::Null, Value::from(r#"Location:"#), Value::Null, Value::Null]).row(vec![Value::from(r#"BrowseDlg"#), Value::from(r#"Description"#), Value::from(r#"Text"#), Value::from(25), Value::from(23), Value::from(280), Value::from(15), Value::from(196611), Value::Null, Value::from(r#"Browse to the destination folder"#), Value::Null, Value::Null]).row(vec![Value::from(r#"BrowseDlg"#), Value::from(r#"Title"#), Value::from(r#"Text"#), Value::from(15), Value::from(6), Value::from(200), Value::from(15), Value::from(196611), Value::Null, Value::from(r#"{\WixUI_Font_Title}Change destination folder"#), Value::Null, Value::Null]).row(vec![Value::from(r#"BrowseDlg"#), Value::from(r#"WixUI_Bmp_Up"#), Value::from(r#"PushButton"#), Value::from(298), Value::from(55), Value::from(19), Value::from(19), Value::from(3670019), Value::Null, Value::from(r#"WixUI_Bmp_Up"#), Value::from(r#"NewFolder"#), Value::from(r#"Up one level|"#)]).row(vec![Value::from(r#"BrowseDlg"#), Value::from("Cancel"), Value::from(r#"PushButton"#), Value::from(304), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from("Cancel"), Value::from(r#"ComboLabel"#), Value::Null]).row(vec![Value::from(r#"BrowseDlg"#), Value::from("OK"), Value::from(r#"PushButton"#), Value::from(240), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from("OK"), Value::from("Cancel"), Value::Null]).row(vec![Value::from(r#"BrowseDlg"#), Value::from(r#"BottomLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(234), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"BrowseDlg"#), Value::from(r#"BannerBitmap"#), Value::from(r#"Bitmap"#), Value::from(0), Value::from(0), Value::from(370), Value::from(44), Value::from(1), Value::Null, Value::from(r#"WixUI_Bmp_Banner"#), Value::from("PathEdit"), Value::Null]).row(vec![Value::from(r#"BrowseDlg"#), Value::from(r#"BannerLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(44), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"BrowseDlg"#), Value::from("PathEdit"), Value::from("PathEdit"), Value::from(25), Value::from(202), Value::from(320), Value::from(18), Value::from(11), Value::from(r#"_BrowseProperty"#), Value::Null, Value::from("OK"), Value::Null]).row(vec![Value::from(r#"BrowseDlg"#), Value::from(r#"ComboLabel"#), Value::from(r#"Text"#), Value::from(25), Value::from(58), Value::from(44), Value::from(10), Value::from(3), Value::Null, Value::from(r#"&Look in:"#), Value::from(r#"DirectoryCombo"#), Value::Null]).row(vec![Value::from(r#"BrowseDlg"#), Value::from(r#"DirectoryCombo"#), Value::from(r#"DirectoryCombo"#), Value::from(70), Value::from(55), Value::from(220), Value::from(80), Value::from(393227), Value::from(r#"_BrowseProperty"#), Value::Null, Value::from(r#"WixUI_Bmp_Up"#), Value::Null]).row(vec![Value::from(r#"BrowseDlg"#), Value::from(r#"NewFolder"#), Value::from(r#"PushButton"#), Value::from(325), Value::from(55), Value::from(19), Value::from(19), Value::from(3670019), Value::Null, Value::from(r#"WixUI_Bmp_New"#), Value::from(r#"DirectoryList"#), Value::from(r#"Create a new folder|"#)]).row(vec![Value::from(r#"BrowseDlg"#), Value::from(r#"DirectoryList"#), Value::from(r#"DirectoryList"#), Value::from(25), Value::from(83), Value::from(320), Value::from(98), Value::from(15), Value::from(r#"_BrowseProperty"#), Value::Null, Value::from(r#"PathLabel"#), Value::Null]).row(vec![Value::from(r#"BrowseDlg"#), Value::from(r#"PathLabel"#), Value::from(r#"Text"#), Value::from(25), Value::from(190), Value::from(320), Value::from(10), Value::from(3), Value::Null, Value::from(r#"&Folder name:"#), Value::from(r#"BannerBitmap"#), Value::Null]).row(vec![Value::from(r#"DiskCostDlg"#), Value::from(r#"Description"#), Value::from(r#"Text"#), Value::from(20), Value::from(20), Value::from(280), Value::from(20), Value::from(196611), Value::Null, Value::from(r#"The disk space required for the installation of the selected features."#), Value::Null, Value::Null]).row(vec![Value::from(r#"DiskCostDlg"#), Value::from(r#"Text"#), Value::from(r#"Text"#), Value::from(20), Value::from(53), Value::from(330), Value::from(50), Value::from(3), Value::Null, Value::from(r#"Highlighted volumes do not have enough disk space available for selected features. You can either remove some files from the highlighted volumes, install fewer features, or select different destination drives."#), Value::Null, Value::Null]).row(vec![Value::from(r#"DiskCostDlg"#), Value::from(r#"Title"#), Value::from(r#"Text"#), Value::from(15), Value::from(6), Value::from(200), Value::from(15), Value::from(196611), Value::Null, Value::from(r#"{\WixUI_Font_Title}Disk Space Requirements"#), Value::Null, Value::Null]).row(vec![Value::from(r#"DiskCostDlg"#), Value::from("OK"), Value::from(r#"PushButton"#), Value::from(304), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from("OK"), Value::from(r#"BannerBitmap"#), Value::Null]).row(vec![Value::from(r#"DiskCostDlg"#), Value::from(r#"BottomLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(234), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"DiskCostDlg"#), Value::from(r#"BannerBitmap"#), Value::from(r#"Bitmap"#), Value::from(0), Value::from(0), Value::from(370), Value::from(44), Value::from(1), Value::Null, Value::from(r#"WixUI_Bmp_Banner"#), Value::from("OK"), Value::Null]).row(vec![Value::from(r#"DiskCostDlg"#), Value::from(r#"BannerLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(44), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"DiskCostDlg"#), Value::from(r#"VolumeList"#), Value::from(r#"VolumeCostList"#), Value::from(20), Value::from(100), Value::from(330), Value::from(120), Value::from(393223), Value::Null, Value::from(r#"{120}{70}{70}{70}{70}"#), Value::Null, Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Remove"#), Value::from(r#"PushButton"#), Value::from(212), Value::from(243), Value::from(80), Value::from(17), Value::from(8388608), Value::Null, Value::from(r#"&Remove"#), Value::from(r#"RemoveNoShield"#), Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Cancel"), Value::from(r#"PushButton"#), Value::from(304), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from("Cancel"), Value::from(r#"Back"#), Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Back"#), Value::from(r#"PushButton"#), Value::from(156), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&Back"#), Value::from(r#"BannerBitmap"#), Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"BottomLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(234), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"BannerBitmap"#), Value::from(r#"Bitmap"#), Value::from(0), Value::from(0), Value::from(370), Value::from(44), Value::from(1), Value::Null, Value::from(r#"WixUI_Bmp_Banner"#), Value::from("Install"), Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"BannerLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(44), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Install"), Value::from(r#"PushButton"#), Value::from(212), Value::from(243), Value::from(80), Value::from(17), Value::from(8388608), Value::Null, Value::from(r#"&Install"#), Value::from("InstallNoShield"), Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("InstallNoShield"), Value::from(r#"PushButton"#), Value::from(212), Value::from(243), Value::from(80), Value::from(17), Value::from(0), Value::Null, Value::from(r#"&Install"#), Value::from(r#"Change"#), Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Change"#), Value::from(r#"PushButton"#), Value::from(212), Value::from(243), Value::from(80), Value::from(17), Value::from(8388608), Value::Null, Value::from(r#"&Change"#), Value::from(r#"ChangeNoShield"#), Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"ChangeNoShield"#), Value::from(r#"PushButton"#), Value::from(212), Value::from(243), Value::from(80), Value::from(17), Value::from(0), Value::Null, Value::from(r#"&Change"#), Value::from("Repair"), Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Repair"), Value::from(r#"PushButton"#), Value::from(212), Value::from(243), Value::from(80), Value::from(17), Value::from(0), Value::Null, Value::from(r#"Re&pair"#), Value::from(r#"Remove"#), Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"RemoveNoShield"#), Value::from(r#"PushButton"#), Value::from(212), Value::from(243), Value::from(80), Value::from(17), Value::from(0), Value::Null, Value::from(r#"&Remove"#), Value::from(r#"Update"#), Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Update"#), Value::from(r#"PushButton"#), Value::from(212), Value::from(243), Value::from(80), Value::from(17), Value::from(8388608), Value::Null, Value::from(r#"&Update"#), Value::from(r#"UpdateNoShield"#), Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"UpdateNoShield"#), Value::from(r#"PushButton"#), Value::from(212), Value::from(243), Value::from(80), Value::from(17), Value::from(0), Value::Null, Value::from(r#"&Update"#), Value::from("Cancel"), Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"InstallTitle"#), Value::from(r#"Text"#), Value::from(15), Value::from(15), Value::from(300), Value::from(15), Value::from(196610), Value::Null, Value::from(r#"{\WixUI_Font_Title}Ready to install [ProductName]"#), Value::Null, Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"InstallText"#), Value::from(r#"Text"#), Value::from(25), Value::from(70), Value::from(320), Value::from(80), Value::from(2), Value::Null, Value::from(r#"Click Install to begin the installation. Click Back to review or change any of your installation settings. Click Cancel to exit the wizard."#), Value::Null, Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"ChangeTitle"#), Value::from(r#"Text"#), Value::from(15), Value::from(15), Value::from(300), Value::from(15), Value::from(196610), Value::Null, Value::from(r#"{\WixUI_Font_Title}Ready to change [ProductName]"#), Value::Null, Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"ChangeText"#), Value::from(r#"Text"#), Value::from(25), Value::from(70), Value::from(320), Value::from(80), Value::from(2), Value::Null, Value::from(r#"Click Change to begin the installation. Click Back to review or change any of your installation settings. Click Cancel to exit the wizard."#), Value::Null, Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"RepairTitle"#), Value::from(r#"Text"#), Value::from(15), Value::from(15), Value::from(300), Value::from(15), Value::from(196610), Value::Null, Value::from(r#"{\WixUI_Font_Title}Ready to repair [ProductName]"#), Value::Null, Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"RepairText"#), Value::from(r#"Text"#), Value::from(25), Value::from(70), Value::from(320), Value::from(80), Value::from(131074), Value::Null, Value::from(r#"Click Repair to repair the installation of [ProductName]. Click Back to review or change any of your installation settings. Click Cancel to exit the wizard."#), Value::Null, Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"RemoveTitle"#), Value::from(r#"Text"#), Value::from(15), Value::from(15), Value::from(300), Value::from(15), Value::from(196610), Value::Null, Value::from(r#"{\WixUI_Font_Title}Ready to remove [ProductName]"#), Value::Null, Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"RemoveText"#), Value::from(r#"Text"#), Value::from(25), Value::from(70), Value::from(320), Value::from(80), Value::from(131074), Value::Null, Value::from(r#"Click Remove to remove [ProductName] from your computer. Click Back to review or change any of your installation settings. Click Cancel to exit the wizard."#), Value::Null, Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"UpdateTitle"#), Value::from(r#"Text"#), Value::from(15), Value::from(15), Value::from(300), Value::from(15), Value::from(196610), Value::Null, Value::from(r#"{\WixUI_Font_Title}Ready to update [ProductName]"#), Value::Null, Value::Null]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"UpdateText"#), Value::from(r#"Text"#), Value::from(25), Value::from(70), Value::from(320), Value::from(80), Value::from(131074), Value::Null, Value::from(r#"Click Update to update [ProductName] from your computer. Click Back to review or change any of your installation settings. Click Cancel to exit the wizard."#), Value::Null, Value::Null]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"Description"#), Value::from(r#"Text"#), Value::from(25), Value::from(23), Value::from(340), Value::from(20), Value::from(196611), Value::Null, Value::from(r#"Select the operation you wish to perform."#), Value::Null, Value::Null]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"Title"#), Value::from(r#"Text"#), Value::from(15), Value::from(6), Value::from(340), Value::from(15), Value::from(196611), Value::Null, Value::from(r#"{\WixUI_Font_Title}Change, repair, or remove installation"#), Value::Null, Value::Null]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from("Cancel"), Value::from(r#"PushButton"#), Value::from(304), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from("Cancel"), Value::from(r#"BannerBitmap"#), Value::Null]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"Back"#), Value::from(r#"PushButton"#), Value::from(180), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&Back"#), Value::from("Next"), Value::Null]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"BottomLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(234), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"BannerBitmap"#), Value::from(r#"Bitmap"#), Value::from(0), Value::from(0), Value::from(370), Value::from(44), Value::from(1), Value::Null, Value::from(r#"WixUI_Bmp_Banner"#), Value::from("ChangeButton"), Value::Null]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"BannerLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(44), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from("Next"), Value::from(r#"PushButton"#), Value::from(236), Value::from(243), Value::from(56), Value::from(17), Value::from(1), Value::Null, Value::from(r#"&Next"#), Value::from("Cancel"), Value::Null]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"ChangeText"#), Value::from(r#"Text"#), Value::from(60), Value::from(85), Value::from(280), Value::from(20), Value::from(3), Value::Null, Value::from(r#"Lets you change the way features are installed."#), Value::Null, Value::Null]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"RepairText"#), Value::from(r#"Text"#), Value::from(60), Value::from(138), Value::from(280), Value::from(30), Value::from(3), Value::Null, Value::from(r#"Repairs errors in the most recent installation by fixing missing and corrupt files, shortcuts, and registry entries."#), Value::Null, Value::Null]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"RemoveText"#), Value::from(r#"Text"#), Value::from(60), Value::from(191), Value::from(280), Value::from(20), Value::from(131075), Value::Null, Value::from(r#"Removes [ProductName] from your computer."#), Value::Null, Value::Null]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from("ChangeButton"), Value::from(r#"PushButton"#), Value::from(40), Value::from(65), Value::from(80), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&Change"#), Value::from(r#"RepairButton"#), Value::from(r#"Change Installation|"#)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"RepairButton"#), Value::from(r#"PushButton"#), Value::from(40), Value::from(118), Value::from(80), Value::from(17), Value::from(3), Value::Null, Value::from(r#"Re&pair"#), Value::from(r#"RemoveButton"#), Value::from(r#"Repair Installation|"#)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"ChangeDisabledText"#), Value::from(r#"Text"#), Value::from(60), Value::from(85), Value::from(280), Value::from(20), Value::from(131074), Value::Null, Value::from(r#"[ProductName] has no independently selectable features."#), Value::Null, Value::Null]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"RemoveButton"#), Value::from(r#"PushButton"#), Value::from(40), Value::from(171), Value::from(80), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&Remove"#), Value::from(r#"Back"#), Value::from(r#"Remove Installation|"#)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"RepairDisabledText"#), Value::from(r#"Text"#), Value::from(60), Value::from(138), Value::from(280), Value::from(30), Value::from(131074), Value::Null, Value::from(r#"[ProductName] cannot be repaired."#), Value::Null, Value::Null]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"RemoveDisabledText"#), Value::from(r#"Text"#), Value::from(60), Value::from(191), Value::from(280), Value::from(20), Value::from(131074), Value::Null, Value::from(r#"[ProductName] cannot be removed."#), Value::Null, Value::Null]).row(vec![Value::from(r#"MaintenanceWelcomeDlg"#), Value::from(r#"Description"#), Value::from(r#"Text"#), Value::from(135), Value::from(70), Value::from(220), Value::from(60), Value::from(196611), Value::Null, Value::from(r#"The Setup Wizard allows you to change the way [ProductName] features are installed on your computer or to remove it from your computer. Click Next to continue or Cancel to exit the Setup Wizard."#), Value::Null, Value::Null]).row(vec![Value::from(r#"MaintenanceWelcomeDlg"#), Value::from(r#"Title"#), Value::from(r#"Text"#), Value::from(135), Value::from(20), Value::from(220), Value::from(60), Value::from(196611), Value::Null, Value::from(r#"{\WixUI_Font_Bigger}Welcome to the [ProductName] Setup Wizard"#), Value::Null, Value::Null]).row(vec![Value::from(r#"MaintenanceWelcomeDlg"#), Value::from("Cancel"), Value::from(r#"PushButton"#), Value::from(304), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from("Cancel"), Value::from(r#"Bitmap"#), Value::Null]).row(vec![Value::from(r#"MaintenanceWelcomeDlg"#), Value::from(r#"Bitmap"#), Value::from(r#"Bitmap"#), Value::from(0), Value::from(0), Value::from(370), Value::from(234), Value::from(1), Value::Null, Value::from(r#"WixUI_Bmp_Dialog"#), Value::from(r#"Back"#), Value::Null]).row(vec![Value::from(r#"MaintenanceWelcomeDlg"#), Value::from(r#"Back"#), Value::from(r#"PushButton"#), Value::from(180), Value::from(243), Value::from(56), Value::from(17), Value::from(1), Value::Null, Value::from(r#"&Back"#), Value::from("Next"), Value::Null]).row(vec![Value::from(r#"MaintenanceWelcomeDlg"#), Value::from(r#"BottomLine"#), Value::from(r#"Line"#), Value::from(0), Value::from(234), Value::from(370), Value::from(0), Value::from(1), Value::Null, Value::Null, Value::Null, Value::Null]).row(vec![Value::from(r#"MaintenanceWelcomeDlg"#), Value::from("Next"), Value::from(r#"PushButton"#), Value::from(236), Value::from(243), Value::from(56), Value::from(17), Value::from(3), Value::Null, Value::from(r#"&Next"#), Value::from("Cancel"), Value::Null]);
    package.insert_rows(query).unwrap();

    package
        .create_table(CONTROL_CONDITION, vec![
            Column::build("Dialog_")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Control_")
                .primary_key()
                .category(Category::Identifier)
                .string(50),
            Column::build("Action")
                .primary_key()
                .enum_values(&["Default", "Disable", "Enable", "Hide", "Show"])
                .string(20),
            Column::build("Condition")
                .primary_key()
                .category(Category::Condition)
                .string(255),
        ])
        .unwrap();

    let query = Insert::into(CONTROL_CONDITION).row(vec![Value::from(r#"ExitDialog"#), Value::from(r#"OptionalCheckBox"#), Value::from(r#"Show"#), Value::from(r#"WIXUI_EXITDIALOGOPTIONALCHECKBOXTEXT AND NOT Installed"#)]).row(vec![Value::from(r#"ExitDialog"#), Value::from(r#"OptionalText"#), Value::from(r#"Show"#), Value::from(r#"WIXUI_EXITDIALOGOPTIONALTEXT AND NOT Installed"#)]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TextInstalling"#), Value::from(r#"Show"#), Value::from(r#"NOT Installed OR (Installed AND (RESUME OR Preselected) AND NOT PATCH)"#)]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TitleInstalling"#), Value::from(r#"Show"#), Value::from(r#"NOT Installed OR (Installed AND (RESUME OR Preselected) AND NOT PATCH)"#)]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TextChanging"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Change""#)]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TitleChanging"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Change""#)]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TextRepairing"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Repair""#)]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TitleRepairing"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Repair""#)]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TextRemoving"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Remove""#)]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TitleRemoving"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Remove""#)]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TextUpdating"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Update""#)]).row(vec![Value::from(r#"ProgressDlg"#), Value::from(r#"TitleUpdating"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Update""#)]).row(vec![Value::from(r#"ResumeDlg"#), Value::from("Install"), Value::from(r#"Show"#), Value::from(r#"ALLUSERS"#)]).row(vec![Value::from(r#"ResumeDlg"#), Value::from("InstallNoShield"), Value::from(r#"Show"#), Value::from(r#"NOT ALLUSERS"#)]).row(vec![Value::from(r#"WelcomeDlg"#), Value::from(r#"Description"#), Value::from(r#"Show"#), Value::from(r#"NOT Installed OR NOT PATCH"#)]).row(vec![Value::from(r#"WelcomeDlg"#), Value::from(r#"Description"#), Value::from(r#"Hide"#), Value::from(r#"Installed AND PATCH"#)]).row(vec![Value::from(r#"WelcomeDlg"#), Value::from(r#"PatchDescription"#), Value::from(r#"Show"#), Value::from(r#"Installed AND PATCH"#)]).row(vec![Value::from(r#"WelcomeDlg"#), Value::from(r#"PatchDescription"#), Value::from(r#"Hide"#), Value::from(r#"NOT Installed OR NOT PATCH"#)]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"Browse"#), Value::from(r#"Hide"#), Value::from(r#"Installed"#)]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"Browse"#), Value::from(r#"Disable"#), Value::from(r#"Installed"#)]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"Location"#), Value::from(r#"Hide"#), Value::from(r#"Installed"#)]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"LocationLabel"#), Value::from(r#"Hide"#), Value::from(r#"Installed"#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Remove"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Remove" AND ALLUSERS"#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Remove"#), Value::from(r#"Enable"#), Value::from(r#"WixUI_InstallMode = "Remove""#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Back"#), Value::from(r#"Default"#), Value::from(r#"WixUI_InstallMode = "Remove""#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Install"), Value::from(r#"Show"#), Value::from(r#"NOT Installed AND ALLUSERS"#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Install"), Value::from(r#"Enable"#), Value::from(r#"NOT Installed"#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Install"), Value::from(r#"Default"#), Value::from(r#"NOT Installed"#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("InstallNoShield"), Value::from(r#"Show"#), Value::from(r#"NOT Installed AND NOT ALLUSERS"#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("InstallNoShield"), Value::from(r#"Enable"#), Value::from(r#"NOT Installed"#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("InstallNoShield"), Value::from(r#"Default"#), Value::from(r#"NOT Installed"#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Change"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Change" AND ALLUSERS AND (ADDLOCAL OR REMOVE)"#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Change"#), Value::from(r#"Enable"#), Value::from(r#"WixUI_InstallMode = "Change""#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Change"#), Value::from(r#"Default"#), Value::from(r#"WixUI_InstallMode = "Change""#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"ChangeNoShield"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Change" AND (NOT ALLUSERS OR (NOT ADDLOCAL AND NOT REMOVE))"#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"ChangeNoShield"#), Value::from(r#"Enable"#), Value::from(r#"WixUI_InstallMode = "Change""#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"ChangeNoShield"#), Value::from(r#"Default"#), Value::from(r#"WixUI_InstallMode = "Change""#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Repair"), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Repair""#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Repair"), Value::from(r#"Enable"#), Value::from(r#"WixUI_InstallMode = "Repair""#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Repair"), Value::from(r#"Default"#), Value::from(r#"WixUI_InstallMode = "Repair""#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"RemoveNoShield"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Remove" AND NOT ALLUSERS"#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"RemoveNoShield"#), Value::from(r#"Enable"#), Value::from(r#"WixUI_InstallMode = "Remove""#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Update"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Update" AND ALLUSERS"#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Update"#), Value::from(r#"Enable"#), Value::from(r#"WixUI_InstallMode = "Update""#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"UpdateNoShield"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Update" AND NOT ALLUSERS"#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"UpdateNoShield"#), Value::from(r#"Enable"#), Value::from(r#"WixUI_InstallMode = "Update""#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"InstallTitle"#), Value::from(r#"Show"#), Value::from(r#"NOT Installed"#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"InstallText"#), Value::from(r#"Show"#), Value::from(r#"NOT Installed"#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"ChangeTitle"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Change""#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"ChangeText"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Change""#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"RepairTitle"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Repair""#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"RepairText"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Repair""#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"RemoveTitle"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Remove""#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"RemoveText"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Remove""#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"UpdateTitle"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Update""#)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"UpdateText"#), Value::from(r#"Show"#), Value::from(r#"WixUI_InstallMode = "Update""#)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"ChangeText"#), Value::from(r#"Hide"#), Value::from(r#"ARPNOMODIFY"#)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"RepairText"#), Value::from(r#"Hide"#), Value::from(r#"ARPNOREPAIR"#)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"RemoveText"#), Value::from(r#"Hide"#), Value::from(r#"ARPNOREMOVE"#)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from("ChangeButton"), Value::from(r#"Disable"#), Value::from(r#"ARPNOMODIFY"#)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"RepairButton"#), Value::from(r#"Disable"#), Value::from(r#"ARPNOREPAIR"#)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"ChangeDisabledText"#), Value::from(r#"Show"#), Value::from(r#"ARPNOMODIFY"#)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"RemoveButton"#), Value::from(r#"Disable"#), Value::from(r#"ARPNOREMOVE"#)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"RepairDisabledText"#), Value::from(r#"Show"#), Value::from(r#"ARPNOREPAIR"#)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"RemoveDisabledText"#), Value::from(r#"Show"#), Value::from(r#"ARPNOREMOVE"#)]);
    package.insert_rows(query).unwrap();

    package
        .create_table(CONTROL_EVENT, vec![
            Column::build("Dialog_")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Control_")
                .primary_key()
                .category(Category::Identifier)
                .string(50),
            Column::build("Event")
                .primary_key()
                .category(Category::Formatted)
                .string(20),
            Column::build("Argument")
                .primary_key()
                .category(Category::Formatted)
                .string(255),
            Column::build("Condition")
                .primary_key()
                .category(Category::Condition)
                .nullable()
                .string(255),
            Column::build("Ordering")
                .category(Category::Condition)
                .nullable()
                .range(0, 2147483647)
                .int16(),
        ])
        .unwrap();

    let query = Insert::into(CONTROL_EVENT).row(vec![Value::from(r#"FatalError"#), Value::from("Finish"), Value::from(r#"EndDialog"#), Value::from(r#"Exit"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"UserExit"#), Value::from("Finish"), Value::from(r#"EndDialog"#), Value::from(r#"Exit"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"ExitDialog"#), Value::from("Finish"), Value::from(r#"DoAction"#), Value::from(r#"LaunchApplication"#), Value::from(r#"WIXUI_EXITDIALOGOPTIONALCHECKBOX = 1 and NOT Installed"#), Value::from(1)]).row(vec![Value::from(r#"ExitDialog"#), Value::from("Finish"), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"1"#), Value::from(999)]).row(vec![Value::from(r#"ErrorDlg"#), Value::from(r#"N"#), Value::from(r#"EndDialog"#), Value::from(r#"ErrorNo"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"ErrorDlg"#), Value::from(r#"Y"#), Value::from(r#"EndDialog"#), Value::from(r#"ErrorYes"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"ErrorDlg"#), Value::from(r#"A"#), Value::from(r#"EndDialog"#), Value::from(r#"ErrorAbort"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"ErrorDlg"#), Value::from(r#"C"#), Value::from(r#"EndDialog"#), Value::from(r#"ErrorCancel"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"ErrorDlg"#), Value::from(r#"I"#), Value::from(r#"EndDialog"#), Value::from(r#"ErrorIgnore"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"ErrorDlg"#), Value::from(r#"O"#), Value::from(r#"EndDialog"#), Value::from(r#"ErrorOk"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"ErrorDlg"#), Value::from(r#"R"#), Value::from(r#"EndDialog"#), Value::from(r#"ErrorRetry"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"FilesInUse"#), Value::from("Retry"), Value::from(r#"EndDialog"#), Value::from("Retry"), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"FilesInUse"#), Value::from(r#"Ignore"#), Value::from(r#"EndDialog"#), Value::from(r#"Ignore"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"FilesInUse"#), Value::from(r#"Exit"#), Value::from(r#"EndDialog"#), Value::from(r#"Exit"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"MsiRMFilesInUse"#), Value::from("Cancel"), Value::from(r#"EndDialog"#), Value::from(r#"Exit"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"MsiRMFilesInUse"#), Value::from("OK"), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"MsiRMFilesInUse"#), Value::from("OK"), Value::from(r#"RMShutdownAndRestart"#), Value::from(r#"0"#), Value::from(r#"WixUIRMOption~="UseRM""#), Value::from(2)]).row(vec![Value::from(r#"PrepareDlg"#), Value::from("Cancel"), Value::from(r#"SpawnDialog"#), Value::from(r#"CancelDlg"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"CancelDlg"#), Value::from("No"), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"CancelDlg"#), Value::from(r#"Yes"#), Value::from(r#"EndDialog"#), Value::from(r#"Exit"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"ProgressDlg"#), Value::from("Cancel"), Value::from(r#"SpawnDialog"#), Value::from(r#"CancelDlg"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"ResumeDlg"#), Value::from("Cancel"), Value::from(r#"SpawnDialog"#), Value::from(r#"CancelDlg"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"ResumeDlg"#), Value::from("Install"), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace <> 1"#), Value::from(2)]).row(vec![Value::from(r#"ResumeDlg"#), Value::from("Install"), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(4)]).row(vec![Value::from(r#"ResumeDlg"#), Value::from("Install"), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND (PROMPTROLLBACKCOST="P" OR NOT PROMPTROLLBACKCOST)"#), Value::from(3)]).row(vec![Value::from(r#"ResumeDlg"#), Value::from("Install"), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfDiskDlg"#), Value::from(r#"(OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 1) OR (OutOfDiskSpace = 1 AND PROMPTROLLBACKCOST="F")"#), Value::from(6)]).row(vec![Value::from(r#"ResumeDlg"#), Value::from("Install"), Value::from(r#"SpawnWaitDialog"#), Value::from(r#"WaitForCostingDlg"#), Value::from(r#"1 OR CostingComplete = 1"#), Value::from(1)]).row(vec![Value::from(r#"ResumeDlg"#), Value::from("Install"), Value::from(r#"EnableRollback"#), Value::from(r#"False"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(5)]).row(vec![Value::from(r#"ResumeDlg"#), Value::from("InstallNoShield"), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace <> 1"#), Value::from(2)]).row(vec![Value::from(r#"ResumeDlg"#), Value::from("InstallNoShield"), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(4)]).row(vec![Value::from(r#"ResumeDlg"#), Value::from("InstallNoShield"), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND (PROMPTROLLBACKCOST="P" OR NOT PROMPTROLLBACKCOST)"#), Value::from(3)]).row(vec![Value::from(r#"ResumeDlg"#), Value::from("InstallNoShield"), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfDiskDlg"#), Value::from(r#"(OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 1) OR (OutOfDiskSpace = 1 AND PROMPTROLLBACKCOST="F")"#), Value::from(6)]).row(vec![Value::from(r#"ResumeDlg"#), Value::from("InstallNoShield"), Value::from(r#"SpawnWaitDialog"#), Value::from(r#"WaitForCostingDlg"#), Value::from(r#"1 OR CostingComplete = 1"#), Value::from(1)]).row(vec![Value::from(r#"ResumeDlg"#), Value::from("InstallNoShield"), Value::from(r#"EnableRollback"#), Value::from(r#"False"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(5)]).row(vec![Value::from(r#"WaitForCostingDlg"#), Value::from("Return"), Value::from(r#"EndDialog"#), Value::from(r#"Exit"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"OutOfRbDiskDlg"#), Value::from("No"), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"Yes"#), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"1"#), Value::from(2)]).row(vec![Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"Yes"#), Value::from(r#"EnableRollback"#), Value::from(r#"False"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"OutOfDiskDlg"#), Value::from("OK"), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"WelcomeDlg"#), Value::from("Cancel"), Value::from(r#"SpawnDialog"#), Value::from(r#"CancelDlg"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"WelcomeDlg"#), Value::from("Next"), Value::from(r#"NewDialog"#), Value::from(r#"CustomizeDlg"#), Value::from(r#"NOT Installed"#), Value::from(1)]).row(vec![Value::from(r#"WelcomeDlg"#), Value::from("Next"), Value::from(r#"NewDialog"#), Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Installed AND PATCH"#), Value::from(1)]).row(vec![Value::from(r#"WelcomeDlg"#), Value::from("Next"), Value::from(r#"[WixUI_InstallMode]"#), Value::from(r#"Update"#), Value::from(r#"Installed AND PATCH"#), Value::from(1)]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from("Cancel"), Value::from(r#"SpawnDialog"#), Value::from(r#"CancelDlg"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"Back"#), Value::from(r#"NewDialog"#), Value::from(r#"WelcomeDlg"#), Value::from(r#"NOT Installed"#), Value::from(2)]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"Back"#), Value::from(r#"NewDialog"#), Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"Installed"#), Value::from(1)]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from("Next"), Value::from(r#"NewDialog"#), Value::from(r#"VerifyReadyDlg"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"Browse"#), Value::from(r#"SelectionBrowse"#), Value::from(r#"BrowseDlg"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"Reset"#), Value::from(r#"Reset"#), Value::from(r#"0"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"CustomizeDlg"#), Value::from(r#"DiskCost"#), Value::from(r#"SpawnDialog"#), Value::from(r#"DiskCostDlg"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"BrowseDlg"#), Value::from(r#"WixUI_Bmp_Up"#), Value::from(r#"DirectoryListUp"#), Value::from(r#"0"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"BrowseDlg"#), Value::from("Cancel"), Value::from(r#"Reset"#), Value::from(r#"0"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"BrowseDlg"#), Value::from("Cancel"), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"1"#), Value::from(2)]).row(vec![Value::from(r#"BrowseDlg"#), Value::from("OK"), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"1"#), Value::from(2)]).row(vec![Value::from(r#"BrowseDlg"#), Value::from("OK"), Value::from(r#"SetTargetPath"#), Value::from(r#"[_BrowseProperty]"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"BrowseDlg"#), Value::from(r#"NewFolder"#), Value::from(r#"DirectoryListNew"#), Value::from(r#"0"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"DiskCostDlg"#), Value::from("OK"), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Remove"#), Value::from(r#"Remove"#), Value::from(r#"All"#), Value::from(r#"OutOfDiskSpace <> 1"#), Value::from(1)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Remove"#), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace <> 1"#), Value::from(2)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Remove"#), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(4)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Remove"#), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND (PROMPTROLLBACKCOST="P" OR NOT PROMPTROLLBACKCOST)"#), Value::from(3)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Remove"#), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfDiskDlg"#), Value::from(r#"(OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 1) OR (OutOfDiskSpace = 1 AND PROMPTROLLBACKCOST="F")"#), Value::from(6)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Remove"#), Value::from(r#"EnableRollback"#), Value::from(r#"False"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(5)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Cancel"), Value::from(r#"SpawnDialog"#), Value::from(r#"CancelDlg"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Back"#), Value::from(r#"NewDialog"#), Value::from(r#"WelcomeDlg"#), Value::from(r#"Installed AND PATCH"#), Value::from(3)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Back"#), Value::from(r#"NewDialog"#), Value::from(r#"CustomizeDlg"#), Value::from(r#"NOT Installed OR WixUI_InstallMode = "Change""#), Value::from(1)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Back"#), Value::from(r#"NewDialog"#), Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"Installed AND NOT PATCH"#), Value::from(2)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Install"), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace <> 1"#), Value::from(1)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Install"), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(3)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Install"), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND (PROMPTROLLBACKCOST="P" OR NOT PROMPTROLLBACKCOST)"#), Value::from(2)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Install"), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfDiskDlg"#), Value::from(r#"(OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 1) OR (OutOfDiskSpace = 1 AND PROMPTROLLBACKCOST="F")"#), Value::from(5)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Install"), Value::from(r#"EnableRollback"#), Value::from(r#"False"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(4)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("InstallNoShield"), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace <> 1"#), Value::from(1)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("InstallNoShield"), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(3)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("InstallNoShield"), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND (PROMPTROLLBACKCOST="P" OR NOT PROMPTROLLBACKCOST)"#), Value::from(2)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("InstallNoShield"), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfDiskDlg"#), Value::from(r#"(OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 1) OR (OutOfDiskSpace = 1 AND PROMPTROLLBACKCOST="F")"#), Value::from(5)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("InstallNoShield"), Value::from(r#"EnableRollback"#), Value::from(r#"False"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(4)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Change"#), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace <> 1"#), Value::from(1)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Change"#), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(3)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Change"#), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND (PROMPTROLLBACKCOST="P" OR NOT PROMPTROLLBACKCOST)"#), Value::from(2)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Change"#), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfDiskDlg"#), Value::from(r#"(OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 1) OR (OutOfDiskSpace = 1 AND PROMPTROLLBACKCOST="F")"#), Value::from(5)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Change"#), Value::from(r#"EnableRollback"#), Value::from(r#"False"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(4)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"ChangeNoShield"#), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace <> 1"#), Value::from(1)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"ChangeNoShield"#), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(3)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"ChangeNoShield"#), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND (PROMPTROLLBACKCOST="P" OR NOT PROMPTROLLBACKCOST)"#), Value::from(2)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"ChangeNoShield"#), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfDiskDlg"#), Value::from(r#"(OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 1) OR (OutOfDiskSpace = 1 AND PROMPTROLLBACKCOST="F")"#), Value::from(5)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"ChangeNoShield"#), Value::from(r#"EnableRollback"#), Value::from(r#"False"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(4)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Repair"), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace <> 1"#), Value::from(3)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Repair"), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(5)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Repair"), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND (PROMPTROLLBACKCOST="P" OR NOT PROMPTROLLBACKCOST)"#), Value::from(4)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Repair"), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfDiskDlg"#), Value::from(r#"(OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 1) OR (OutOfDiskSpace = 1 AND PROMPTROLLBACKCOST="F")"#), Value::from(7)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Repair"), Value::from(r#"EnableRollback"#), Value::from(r#"False"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(6)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Repair"), Value::from(r#"ReinstallMode"#), Value::from(r#"ecmus"#), Value::from(r#"OutOfDiskSpace <> 1"#), Value::from(1)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from("Repair"), Value::from(r#"Reinstall"#), Value::from(r#"All"#), Value::from(r#"OutOfDiskSpace <> 1"#), Value::from(2)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"RemoveNoShield"#), Value::from(r#"Remove"#), Value::from(r#"All"#), Value::from(r#"OutOfDiskSpace <> 1"#), Value::from(1)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"RemoveNoShield"#), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace <> 1"#), Value::from(2)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"RemoveNoShield"#), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(4)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"RemoveNoShield"#), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND (PROMPTROLLBACKCOST="P" OR NOT PROMPTROLLBACKCOST)"#), Value::from(3)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"RemoveNoShield"#), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfDiskDlg"#), Value::from(r#"(OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 1) OR (OutOfDiskSpace = 1 AND PROMPTROLLBACKCOST="F")"#), Value::from(6)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"RemoveNoShield"#), Value::from(r#"EnableRollback"#), Value::from(r#"False"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(5)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Update"#), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace <> 1"#), Value::from(1)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Update"#), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(3)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Update"#), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND (PROMPTROLLBACKCOST="P" OR NOT PROMPTROLLBACKCOST)"#), Value::from(2)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Update"#), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfDiskDlg"#), Value::from(r#"(OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 1) OR (OutOfDiskSpace = 1 AND PROMPTROLLBACKCOST="F")"#), Value::from(5)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"Update"#), Value::from(r#"EnableRollback"#), Value::from(r#"False"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(4)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"UpdateNoShield"#), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace <> 1"#), Value::from(1)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"UpdateNoShield"#), Value::from(r#"EndDialog"#), Value::from("Return"), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(3)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"UpdateNoShield"#), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfRbDiskDlg"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND (PROMPTROLLBACKCOST="P" OR NOT PROMPTROLLBACKCOST)"#), Value::from(2)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"UpdateNoShield"#), Value::from(r#"SpawnDialog"#), Value::from(r#"OutOfDiskDlg"#), Value::from(r#"(OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 1) OR (OutOfDiskSpace = 1 AND PROMPTROLLBACKCOST="F")"#), Value::from(5)]).row(vec![Value::from(r#"VerifyReadyDlg"#), Value::from(r#"UpdateNoShield"#), Value::from(r#"EnableRollback"#), Value::from(r#"False"#), Value::from(r#"OutOfDiskSpace = 1 AND OutOfNoRbDiskSpace = 0 AND PROMPTROLLBACKCOST="D""#), Value::from(4)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from("Cancel"), Value::from(r#"SpawnDialog"#), Value::from(r#"CancelDlg"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"Back"#), Value::from(r#"NewDialog"#), Value::from(r#"MaintenanceWelcomeDlg"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from("ChangeButton"), Value::from(r#"NewDialog"#), Value::from(r#"CustomizeDlg"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from("ChangeButton"), Value::from(r#"[WixUI_InstallMode]"#), Value::from(r#"Change"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"RepairButton"#), Value::from(r#"NewDialog"#), Value::from(r#"VerifyReadyDlg"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"RepairButton"#), Value::from(r#"[WixUI_InstallMode]"#), Value::from("Repair"), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"RemoveButton"#), Value::from(r#"NewDialog"#), Value::from(r#"VerifyReadyDlg"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"RemoveButton"#), Value::from(r#"[WixUI_InstallMode]"#), Value::from(r#"Remove"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"MaintenanceWelcomeDlg"#), Value::from("Cancel"), Value::from(r#"SpawnDialog"#), Value::from(r#"CancelDlg"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"MaintenanceWelcomeDlg"#), Value::from("Next"), Value::from(r#"NewDialog"#), Value::from(r#"MaintenanceTypeDlg"#), Value::from(r#"1"#), Value::from(1)]).row(vec![Value::from(r#"MaintenanceWelcomeDlg"#), Value::from("Next"), Value::from(r#"SpawnWaitDialog"#), Value::from(r#"WaitForCostingDlg"#), Value::from(r#"1 OR CostingComplete = 1"#), Value::from(1)]);
    package.insert_rows(query).unwrap();

    package
        .create_table(CREATE_FOLDER, vec![
            Column::build("Directory_")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Component_")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
        ])
        .unwrap();

    let query = Insert::into(CREATE_FOLDER)
        .row(vec![Value::from("APPLICATIONFOLDER"), Value::from("Path")]);
    package.insert_rows(query).unwrap();

    package
        .create_table(CUSTOM_ACTION, vec![
            Column::build("Action")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Type").range(1, 32767).int16(),
            Column::build("Source")
                .nullable()
                .category(Category::CustomSource)
                .string(72),
            Column::build("Target")
                .nullable()
                .category(Category::Formatted)
                .string(255),
            Column::build("ExtendedType")
                .nullable()
                .range(0, 2147483647)
                .int32(),
        ])
        .unwrap();

    let query = Insert::into(CUSTOM_ACTION)
        .row(vec![
            Value::from("LaunchApplication"),
            Value::from(1),
            Value::from("WixCA"),
            Value::from("WixShellExec"),
            Value::Null,
        ])
        .row(vec![
            Value::from("SetARPINSTALLLOCATION"),
            Value::from(51),
            Value::from("ARPINSTALLLOCATION"),
            Value::from("[APPLICATIONFOLDER]"),
            Value::Null,
        ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(DIALOG, vec![
            Column::build("Dialog")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("HCentering").range(0, 100).int16(),
            Column::build("VCentering").range(0, 100).int16(),
            Column::build("Width").range(0, 32767).int16(),
            Column::build("Height").range(0, 32767).int16(),
            Column::build("Attributes")
                .nullable()
                .range(0, 2147483647)
                .int32(),
            Column::build("Title")
                .nullable()
                .localizable()
                .category(Category::Formatted)
                .string(128),
            Column::build("Control_First")
                .category(Category::Identifier)
                .string(50),
            Column::build("Control_Default")
                .nullable()
                .category(Category::Identifier)
                .string(50),
            Column::build("Control_Cancel")
                .nullable()
                .category(Category::Identifier)
                .string(50),
        ])
        .unwrap();

    let query = Insert::into(DIALOG)
        .row(vec![
            Value::from("FatalError"),
            Value::from(50),
            Value::from(50),
            Value::from(370),
            Value::from(270),
            Value::from(7),
            Value::from("[ProductName] Setup"),
            Value::from("Finish"),
            Value::from("Finish"),
            Value::from("Finish"),
        ])
        .row(vec![
            Value::from("UserExit"),
            Value::from(50),
            Value::from(50),
            Value::from(370),
            Value::from(270),
            Value::from(7),
            Value::from("[ProductName] Setup"),
            Value::from("Finish"),
            Value::from("Finish"),
            Value::from("Finish"),
        ])
        .row(vec![
            Value::from("ExitDialog"),
            Value::from(50),
            Value::from(50),
            Value::from(370),
            Value::from(270),
            Value::from(7),
            Value::from("[ProductName] Setup"),
            Value::from("Finish"),
            Value::from("Finish"),
            Value::from("Finish"),
        ])
        .row(vec![
            Value::from("ErrorDlg"),
            Value::from(50),
            Value::from(50),
            Value::from(270),
            Value::from(105),
            Value::from(65543),
            Value::from("[ProductName] Setup"),
            Value::from("ErrorText"),
            Value::Null,
            Value::Null,
        ])
        .row(vec![
            Value::from("FilesInUse"),
            Value::from(50),
            Value::from(50),
            Value::from(370),
            Value::from(270),
            Value::from(23),
            Value::from("[ProductName] Setup"),
            Value::from("Retry"),
            Value::from("Retry"),
            Value::from("Retry"),
        ])
        .row(vec![
            Value::from("MsiRMFilesInUse"),
            Value::from(50),
            Value::from(50),
            Value::from(370),
            Value::from(270),
            Value::from(23),
            Value::from("[ProductName] Setup"),
            Value::from("OK"),
            Value::from("OK"),
            Value::from("Cancel"),
        ])
        .row(vec![
            Value::from("PrepareDlg"),
            Value::from(50),
            Value::from(50),
            Value::from(370),
            Value::from(270),
            Value::from(5),
            Value::from("[ProductName] Setup"),
            Value::from("Cancel"),
            Value::from("Cancel"),
            Value::from("Cancel"),
        ])
        .row(vec![
            Value::from("CancelDlg"),
            Value::from(50),
            Value::from(50),
            Value::from(260),
            Value::from(85),
            Value::from(7),
            Value::from("[ProductName] Setup"),
            Value::from("No"),
            Value::from("No"),
            Value::from("No"),
        ])
        .row(vec![
            Value::from("ProgressDlg"),
            Value::from(50),
            Value::from(50),
            Value::from(370),
            Value::from(270),
            Value::from(5),
            Value::from("[ProductName] Setup"),
            Value::from("Cancel"),
            Value::from("Cancel"),
            Value::from("Cancel"),
        ])
        .row(vec![
            Value::from("ResumeDlg"),
            Value::from(50),
            Value::from(50),
            Value::from(370),
            Value::from(270),
            Value::from(7),
            Value::from("[ProductName] Setup"),
            Value::from("Install"),
            Value::from("InstallNoShield"),
            Value::from("Cancel"),
        ])
        .row(vec![
            Value::from("WaitForCostingDlg"),
            Value::from(50),
            Value::from(50),
            Value::from(260),
            Value::from(85),
            Value::from(7),
            Value::from("[ProductName] Setup"),
            Value::from("Return"),
            Value::from("Return"),
            Value::from("Return"),
        ])
        .row(vec![
            Value::from("OutOfRbDiskDlg"),
            Value::from(50),
            Value::from(50),
            Value::from(370),
            Value::from(270),
            Value::from(7),
            Value::from("[ProductName] Setup"),
            Value::from("No"),
            Value::from("No"),
            Value::from("No"),
        ])
        .row(vec![
            Value::from("OutOfDiskDlg"),
            Value::from(50),
            Value::from(50),
            Value::from(370),
            Value::from(270),
            Value::from(7),
            Value::from("[ProductName] Setup"),
            Value::from("OK"),
            Value::from("OK"),
            Value::from("OK"),
        ])
        .row(vec![
            Value::from("WelcomeDlg"),
            Value::from(50),
            Value::from(50),
            Value::from(370),
            Value::from(270),
            Value::from(7),
            Value::from("[ProductName] Setup"),
            Value::from("Next"),
            Value::from("Next"),
            Value::from("Cancel"),
        ])
        .row(vec![
            Value::from("CustomizeDlg"),
            Value::from(50),
            Value::from(50),
            Value::from(370),
            Value::from(270),
            Value::from(39),
            Value::from("[ProductName] Setup"),
            Value::from("Tree"),
            Value::from("Next"),
            Value::from("Cancel"),
        ])
        .row(vec![
            Value::from("BrowseDlg"),
            Value::from(50),
            Value::from(50),
            Value::from(370),
            Value::from(270),
            Value::from(7),
            Value::from("[ProductName] Setup"),
            Value::from("PathEdit"),
            Value::from("OK"),
            Value::from("Cancel"),
        ])
        .row(vec![
            Value::from("DiskCostDlg"),
            Value::from(50),
            Value::from(50),
            Value::from(370),
            Value::from(270),
            Value::from(7),
            Value::from("[ProductName] Setup"),
            Value::from("OK"),
            Value::from("OK"),
            Value::from("OK"),
        ])
        .row(vec![
            Value::from("VerifyReadyDlg"),
            Value::from(50),
            Value::from(50),
            Value::from(370),
            Value::from(270),
            Value::from(39),
            Value::from("[ProductName] Setup"),
            Value::from("Install"),
            Value::from("Repair"),
            Value::from("Cancel"),
        ])
        .row(vec![
            Value::from("MaintenanceTypeDlg"),
            Value::from(50),
            Value::from(50),
            Value::from(370),
            Value::from(270),
            Value::from(7),
            Value::from("[ProductName] Setup"),
            Value::from("ChangeButton"),
            Value::from("ChangeButton"),
            Value::from("Cancel"),
        ])
        .row(vec![
            Value::from("MaintenanceWelcomeDlg"),
            Value::from(50),
            Value::from(50),
            Value::from(370),
            Value::from(270),
            Value::from(7),
            Value::from("[ProductName] Setup"),
            Value::from("Next"),
            Value::from("Next"),
            Value::from("Cancel"),
        ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(DIRECTORY, vec![
            Column::build("Directory")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Directory_Parent")
                .nullable()
                .category(Category::Identifier)
                .string(72),
            Column::build("DefaultDir")
                .localizable()
                .category(Category::DefaultDir)
                .string(255),
        ])
        .unwrap();

    let query = Insert::into(DIRECTORY)
        .row(vec![
            Value::from("APPLICATIONFOLDER"),
            Value::from("ProgramFiles64Folder"),
            Value::from("aimrk17e|Airshipper"),
        ])
        .row(vec![
            Value::from("ApplicationProgramsFolder"),
            Value::from("ProgramMenuFolder"),
            Value::from("spdn7ech|Airshipper"),
        ])
        .row(vec![
            Value::from("DesktopFolder"),
            Value::from("TARGETDIR"),
            Value::from("Desktop"),
        ])
        .row(vec![
            Value::from("ProgramMenuFolder"),
            Value::from("TARGETDIR"),
            Value::from("."),
        ])
        .row(vec![
            Value::from("TARGETDIR"),
            Value::Null,
            Value::from("SourceDir"),
        ])
        .row(vec![
            Value::from("ProgramFiles64Folder"),
            Value::from("TARGETDIR"),
            Value::from("PFiles"),
        ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(ENVIRONMENT, vec![
            Column::build("Environment")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Name")
                .localizable()
                .category(Category::Text)
                .string(255),
            Column::build("Value")
                .localizable()
                .nullable()
                .category(Category::Formatted)
                .string(255),
            Column::build("Component_")
                .category(Category::Identifier)
                .string(72),
        ])
        .unwrap();

    let query = Insert::into(ENVIRONMENT).row(vec![
        Value::from("PATH"),
        Value::from("=-*PATH"),
        Value::from("[~];[APPLICATIONFOLDER]"),
        Value::from("Path"),
    ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(EVENT_MAPPING, vec![
            Column::build("Dialog_")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Control_")
                .primary_key()
                .category(Category::Identifier)
                .string(50),
            Column::build("Event")
                .primary_key()
                .category(Category::Identifier)
                .string(50),
            Column::build("Attribute")
                .category(Category::Identifier)
                .string(50),
        ])
        .unwrap();

    let query = Insert::into(EVENT_MAPPING)
        .row(vec![
            Value::from("PrepareDlg"),
            Value::from("ActionData"),
            Value::from("ActionData"),
            Value::from("Text"),
        ])
        .row(vec![
            Value::from("PrepareDlg"),
            Value::from("ActionText"),
            Value::from("ActionText"),
            Value::from("Text"),
        ])
        .row(vec![
            Value::from("ProgressDlg"),
            Value::from("ActionText"),
            Value::from("ActionText"),
            Value::from("Text"),
        ])
        .row(vec![
            Value::from("ProgressDlg"),
            Value::from("ProgressBar"),
            Value::from("SetProgress"),
            Value::from("Progress"),
        ])
        .row(vec![
            Value::from("CustomizeDlg"),
            Value::from("Next"),
            Value::from("SelectionNoItems"),
            Value::from("Enabled"),
        ])
        .row(vec![
            Value::from("CustomizeDlg"),
            Value::from("Reset"),
            Value::from("SelectionNoItems"),
            Value::from("Enabled"),
        ])
        .row(vec![
            Value::from("CustomizeDlg"),
            Value::from("DiskCost"),
            Value::from("SelectionNoItems"),
            Value::from("Enabled"),
        ])
        .row(vec![
            Value::from("CustomizeDlg"),
            Value::from("ItemDescription"),
            Value::from("SelectionDescription"),
            Value::from("Text"),
        ])
        .row(vec![
            Value::from("CustomizeDlg"),
            Value::from("ItemSize"),
            Value::from("SelectionSize"),
            Value::from("Text"),
        ])
        .row(vec![
            Value::from("CustomizeDlg"),
            Value::from("Location"),
            Value::from("SelectionPath"),
            Value::from("Text"),
        ])
        .row(vec![
            Value::from("CustomizeDlg"),
            Value::from("Location"),
            Value::from("SelectionPathOn"),
            Value::from("Visible"),
        ])
        .row(vec![
            Value::from("CustomizeDlg"),
            Value::from("LocationLabel"),
            Value::from("SelectionPathOn"),
            Value::from("Visible"),
        ])
        .row(vec![
            Value::from("BrowseDlg"),
            Value::from("DirectoryCombo"),
            Value::from("IgnoreChange"),
            Value::from("IgnoreChange"),
        ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(FEATURE, vec![
            Column::build("Feature")
                .primary_key()
                .category(Category::Identifier)
                .string(38),
            Column::build("Feature_Parent")
                .nullable()
                .category(Category::Identifier)
                .string(38),
            Column::build("Title")
                .localizable()
                .nullable()
                .category(Category::Text)
                .string(64),
            Column::build("Description")
                .localizable()
                .nullable()
                .category(Category::Text)
                .string(255),
            Column::build("Display").nullable().range(0, 32767).int16(),
            Column::build("Level").range(0, 32767).int16(),
            Column::build("Directory_")
                .nullable()
                .category(Category::UpperCase)
                .string(72),
            Column::build("Attributes")
                .enum_values(&[
                    "0", "1", "2", "4", "5", "6", "8", "9", "10", "16", "17", "18", "20",
                    "21", "22", "24", "25", "26", "32", "33", "34", "36", "37", "38",
                    "48", "49", "50", "52", "53", "54",
                ])
                .int16(),
        ])
        .unwrap();

    let query = Insert::into(FEATURE)
        .row(vec![
            Value::from("Environment"),
            Value::from("MainProgram"),
            Value::from("PATH Environment Variable"),
            Value::from(
                "Add the install location of the [ProductName] executable to the PATH \
                 system environment variable. This allows the [ProductName] executable \
                 to be called from any location.",
            ),
            Value::from(2),
            Value::from(1),
            Value::Null,
            Value::from(0),
        ])
        .row(vec![
            Value::from("MainProgram"),
            Value::Null,
            Value::from("Airshipper"),
            Value::from("Installs Airshipper."),
            Value::from(1),
            Value::from(1),
            Value::from("APPLICATIONFOLDER"),
            Value::from(24),
        ])
        .row(vec![
            Value::from("DesktopShortcut"),
            Value::from("MainProgram"),
            Value::from("Add a Desktop shortcut"),
            Value::from("Creates a shortcut on your desktop for easier access."),
            Value::from(4),
            Value::from(1),
            Value::Null,
            Value::from(0),
        ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(FEATURE_COMPONENTS, vec![
            Column::build("Feature_")
                .primary_key()
                .category(Category::Identifier)
                .string(38),
            Column::build("Component_")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
        ])
        .unwrap();

    let query = Insert::into(FEATURE_COMPONENTS)
        .row(vec![Value::from("Environment"), Value::from("Path")])
        .row(vec![
            Value::from("MainProgram"),
            Value::from("ApplicationShortcut"),
        ])
        .row(vec![
            Value::from("MainProgram"),
            Value::from("airshipper.exe"),
        ])
        .row(vec![
            Value::from("DesktopShortcut"),
            Value::from("ApplicationShortcutDesktop"),
        ])
        .row(vec![
            Value::from("DesktopShortcut"),
            Value::from("ApplicationShortcutCompatibilityDesktop"),
        ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(FILE, vec![
            Column::build("File")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Component_")
                .category(Category::Identifier)
                .string(72),
            Column::build("FileName")
                .localizable()
                .category(Category::Filename)
                .string(255),
            Column::build("FileSize").range(0, 2147483647).int32(),
            Column::build("Version")
                .nullable()
                .category(Category::Version)
                .string(72),
            Column::build("Language")
                .nullable()
                .category(Category::Language)
                .string(20),
            Column::build("Attributes")
                .nullable()
                .range(0, 32767)
                .int16(),
            Column::build("Sequence").range(1, 2147483647).int32(),
        ])
        .unwrap();

    let query = Insert::into(FILE).row(vec![
        Value::from("airshipper.exe"),
        Value::from("airshipper.exe"),
        Value::from("edzqr9kj.exe|airshipper.exe"),
        Value::from(30805504),
        Value::from(format!("{version}.0")),
        Value::from("0"),
        Value::from(512),
        Value::from(1),
    ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(ICON, vec![
            Column::build("Name")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Data").category(Category::Binary).string(0),
        ])
        .unwrap();

    let query =
        Insert::into(ICON).row(vec![Value::from("ProductICO"), Value::from("Name")]);
    package.insert_rows(query).unwrap();

    package
        .create_table(INSTALL_EXECUTE_SEQUENCE, vec![
            Column::build("Action")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Condition")
                .nullable()
                .category(Category::Condition)
                .string(255),
            Column::build("Sequence")
                .nullable()
                .range(-4, 32767)
                .int16(),
        ])
        .unwrap();

    let query = Insert::into(INSTALL_EXECUTE_SEQUENCE)
        .row(vec![Value::from("AppSearch"), Value::Null, Value::from(50)])
        .row(vec![
            Value::from("CostInitialize"),
            Value::Null,
            Value::from(800),
        ])
        .row(vec![Value::from("FileCost"), Value::Null, Value::from(900)])
        .row(vec![
            Value::from("CostFinalize"),
            Value::Null,
            Value::from(1000),
        ])
        .row(vec![
            Value::from("InstallValidate"),
            Value::Null,
            Value::from(1400),
        ])
        .row(vec![
            Value::from("InstallInitialize"),
            Value::Null,
            Value::from(1500),
        ])
        .row(vec![
            Value::from("InstallFiles"),
            Value::Null,
            Value::from(4000),
        ])
        .row(vec![
            Value::from("InstallFinalize"),
            Value::Null,
            Value::from(6600),
        ])
        .row(vec![
            Value::from("CreateShortcuts"),
            Value::Null,
            Value::from(4500),
        ])
        .row(vec![
            Value::from("PublishFeatures"),
            Value::Null,
            Value::from(6300),
        ])
        .row(vec![
            Value::from("PublishProduct"),
            Value::Null,
            Value::from(6400),
        ])
        .row(vec![
            Value::from("LaunchApplication"),
            Value::from("AUTOSTART = 1"),
            Value::from(6601),
        ])
        .row(vec![
            Value::from("SetARPINSTALLLOCATION"),
            Value::Null,
            Value::from(1001),
        ])
        .row(vec![
            Value::from("FindRelatedProducts"),
            Value::Null,
            Value::from(25),
        ])
        .row(vec![
            Value::from("LaunchConditions"),
            Value::Null,
            Value::from(100),
        ])
        .row(vec![
            Value::from("ValidateProductID"),
            Value::Null,
            Value::from(700),
        ])
        .row(vec![
            Value::from("MigrateFeatureStates"),
            Value::Null,
            Value::from(1200),
        ])
        .row(vec![
            Value::from("ProcessComponents"),
            Value::Null,
            Value::from(1600),
        ])
        .row(vec![
            Value::from("UnpublishFeatures"),
            Value::Null,
            Value::from(1800),
        ])
        .row(vec![
            Value::from("RemoveRegistryValues"),
            Value::Null,
            Value::from(2600),
        ])
        .row(vec![
            Value::from("RemoveShortcuts"),
            Value::Null,
            Value::from(3200),
        ])
        .row(vec![
            Value::from("RemoveEnvironmentStrings"),
            Value::Null,
            Value::from(3300),
        ])
        .row(vec![
            Value::from("RemoveFiles"),
            Value::Null,
            Value::from(3500),
        ])
        .row(vec![
            Value::from("RemoveFolders"),
            Value::Null,
            Value::from(3600),
        ])
        .row(vec![
            Value::from("CreateFolders"),
            Value::Null,
            Value::from(3700),
        ])
        .row(vec![
            Value::from("WriteRegistryValues"),
            Value::Null,
            Value::from(5000),
        ])
        .row(vec![
            Value::from("WriteEnvironmentStrings"),
            Value::Null,
            Value::from(5200),
        ])
        .row(vec![
            Value::from("RegisterUser"),
            Value::Null,
            Value::from(6000),
        ])
        .row(vec![
            Value::from("RegisterProduct"),
            Value::Null,
            Value::from(6100),
        ])
        .row(vec![
            Value::from("RemoveExistingProducts"),
            Value::Null,
            Value::from(1501),
        ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(INSTALL_UI_SEQUENCE, vec![
            Column::build("Action")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Condition")
                .nullable()
                .category(Category::Condition)
                .string(255),
            Column::build("Sequence")
                .nullable()
                .range(-4, 32767)
                .int16(),
        ])
        .unwrap();

    let query = Insert::into(INSTALL_UI_SEQUENCE)
        .row(vec![Value::from("AppSearch"), Value::Null, Value::from(50)])
        .row(vec![
            Value::from("CostInitialize"),
            Value::Null,
            Value::from(800),
        ])
        .row(vec![Value::from("FileCost"), Value::Null, Value::from(900)])
        .row(vec![
            Value::from("CostFinalize"),
            Value::Null,
            Value::from(1000),
        ])
        .row(vec![
            Value::from("FatalError"),
            Value::Null,
            Value::from(-3),
        ])
        .row(vec![Value::from("UserExit"), Value::Null, Value::from(-2)])
        .row(vec![
            Value::from("ExitDialog"),
            Value::Null,
            Value::from(-1),
        ])
        .row(vec![
            Value::from("ExecuteAction"),
            Value::Null,
            Value::from(1300),
        ])
        .row(vec![
            Value::from("PrepareDlg"),
            Value::Null,
            Value::from(49),
        ])
        .row(vec![
            Value::from("ProgressDlg"),
            Value::Null,
            Value::from(1299),
        ])
        .row(vec![
            Value::from("ResumeDlg"),
            Value::from("Installed AND (RESUME OR Preselected)"),
            Value::from(1297),
        ])
        .row(vec![
            Value::from("WelcomeDlg"),
            Value::from("NOT Installed OR PATCH"),
            Value::from(1298),
        ])
        .row(vec![
            Value::from("MaintenanceWelcomeDlg"),
            Value::from("Installed AND NOT RESUME AND NOT Preselected AND NOT PATCH"),
            Value::from(1296),
        ])
        .row(vec![
            Value::from("SetARPINSTALLLOCATION"),
            Value::Null,
            Value::from(1001),
        ])
        .row(vec![
            Value::from("FindRelatedProducts"),
            Value::Null,
            Value::from(25),
        ])
        .row(vec![
            Value::from("LaunchConditions"),
            Value::Null,
            Value::from(100),
        ])
        .row(vec![
            Value::from("ValidateProductID"),
            Value::Null,
            Value::from(700),
        ])
        .row(vec![
            Value::from("MigrateFeatureStates"),
            Value::Null,
            Value::from(1200),
        ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(LAUNCH_CONDITION, vec![
            Column::build("Condition")
                .primary_key()
                .category(Category::Condition)
                .string(255),
            Column::build("Description")
                .localizable()
                .category(Category::Formatted)
                .string(255),
        ])
        .unwrap();

    let query = Insert::into(LAUNCH_CONDITION).row(vec![
        Value::from("NOT WIX_DOWNGRADE_DETECTED"),
        Value::from(
            "A newer version of [ProductName] is already installed. Setup will now exit.",
        ),
    ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(LIST_BOX, vec![
            Column::build("Property")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Order").primary_key().range(1, 32767).int16(),
            Column::build("Value")
                .category(Category::Formatted)
                .string(64),
            Column::build("Text")
                .nullable()
                .localizable()
                .category(Category::Text)
                .string(64),
        ])
        .unwrap();

    // empty

    package
        .create_table(MEDIA, vec![
            Column::build("DiskId")
                .primary_key()
                .range(1, 32767)
                .int16(),
            Column::build("LastSequence").range(0, 2147483647).int32(),
            Column::build("DiskPrompt")
                .nullable()
                .localizable()
                .category(Category::Text)
                .string(64),
            Column::build("Cabinet")
                .nullable()
                .category(Category::Cabinet)
                .string(255),
            Column::build("VolumeLabel")
                .nullable()
                .category(Category::Text)
                .string(32),
            Column::build("Source")
                .nullable()
                .category(Category::Property)
                .string(72),
        ])
        .unwrap();

    let query = Insert::into(MEDIA).row(vec![
        Value::from(1),
        Value::from(1),
        Value::from("CD-ROM #1"),
        Value::from("#media1.cab"),
        Value::Null,
        Value::Null,
    ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(PROPERTY, vec![
            Column::build("Property")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Value")
                .localizable()
                .category(Category::Text)
                .string(0),
        ])
        .unwrap();

    let query = Insert::into(PROPERTY)
        .row(vec![
            Value::from("DiskPrompt"),
            Value::from("Airshipper Installation"),
        ])
        .row(vec![
            Value::from("UpgradeCode"),
            Value::from("{1715788C-2FC7-44D7-912D-2B46202C2FD9}"),
        ])
        .row(vec![
            Value::from("WIXUI_EXITDIALOGOPTIONALCHECKBOX"),
            Value::from("1"),
        ])
        .row(vec![Value::from("WixUIRMOption"), Value::from("UseRM")])
        .row(vec![Value::from("ALLUSERS"), Value::from("1")])
        .row(vec![Value::from("START_VIA_REGISTRY"), Value::from("1")])
        .row(vec![
            Value::from("ARPPRODUCTICON"),
            Value::from("ProductICO"),
        ])
        .row(vec![Value::from("ARPHELPLINK"), Value::from("veloren.net")])
        .row(vec![Value::from("AUTOSTART"), Value::from("0")])
        .row(vec![
            Value::from("WIXUI_EXITDIALOGOPTIONALCHECKBOXTEXT"),
            Value::from("Launch Airshipper"),
        ])
        .row(vec![
            Value::from("WixShellExecTarget"),
            Value::from("[#airshipper.exe]"),
        ])
        .row(vec![
            Value::from("Manufacturer"),
            Value::from("Airshipper contributors"),
        ])
        .row(vec![
            Value::from("ProductCode"),
            Value::from("{2582ED40-3C12-4B01-9DCE-D3079936CE45}"),
        ])
        .row(vec![Value::from("ProductLanguage"), Value::from("1033")])
        .row(vec![Value::from("ProductName"), Value::from("Airshipper")])
        .row(vec![Value::from("ProductVersion"), Value::from(version)])
        .row(vec![
            Value::from("DefaultUIFont"),
            Value::from("WixUI_Font_Normal"),
        ])
        .row(vec![Value::from("WixUI_Mode"), Value::from("FeatureTree")])
        .row(vec![Value::from("ErrorDialog"), Value::from("ErrorDlg")])
        .row(vec![
            Value::from("SecureCustomProperties"),
            Value::from("APPLICATIONFOLDER;WIX_DOWNGRADE_DETECTED;WIX_UPGRADE_DETECTED"),
        ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(RADIO_BUTTON, vec![
            Column::build("Property")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Order").primary_key().range(1, 32767).int16(),
            Column::build("Value")
                .category(Category::Formatted)
                .string(64),
            Column::build("X").range(0, 32767).int16(),
            Column::build("Y").range(0, 32767).int16(),
            Column::build("Width").range(0, 32767).int16(),
            Column::build("Height").range(0, 32767).int16(),
            Column::build("Text")
                .nullable()
                .localizable()
                .category(Category::Text)
                .string(0),
            Column::build("Help")
                .nullable()
                .localizable()
                .category(Category::Text)
                .string(50),
        ])
        .unwrap();

    let query = Insert::into(RADIO_BUTTON)
        .row(vec![
            Value::from("WixUIRMOption"),
            Value::from(1),
            Value::from("UseRM"),
            Value::from(0),
            Value::from(0),
            Value::from(295),
            Value::from(16),
            Value::from("&Close the applications and attempt to restart them."),
            Value::Null,
        ])
        .row(vec![
            Value::from("WixUIRMOption"),
            Value::from(2),
            Value::from("DontUseRM"),
            Value::from(0),
            Value::from(20),
            Value::from(295),
            Value::from(16),
            Value::from("&Do not close applications. A reboot will be required."),
            Value::Null,
        ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(REG_LOCATOR, vec![
            Column::build("Signature_")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Root").range(0, 3).int16(),
            Column::build("Key").category(Category::RegPath).string(255),
            Column::build("Name")
                .nullable()
                .category(Category::Formatted)
                .string(255),
            Column::build("Type").nullable().range(0, 18).int16(),
        ])
        .unwrap();

    let query = Insert::into(REG_LOCATOR).row(vec![
        Value::from(r#"FindInstallLocation"#),
        Value::from(1),
        Value::from(r#"Software\Airshipper"#),
        Value::from(r#"InstallationPath"#),
        Value::from(18),
    ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(REGISTRY, vec![
            Column::build("Registry")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Root").range(-1, 3).int16(),
            Column::build("Key")
                .localizable()
                .category(Category::RegPath)
                .string(255),
            Column::build("Name")
                .nullable()
                .localizable()
                .category(Category::Formatted)
                .string(255),
            Column::build("Value")
                .nullable()
                .localizable()
                .category(Category::Formatted)
                .string(0),
            Column::build("Component_")
                .category(Category::Identifier)
                .string(72),
        ])
        .unwrap();

    let query = Insert::into(REGISTRY)
        .row(vec![
            Value::from(r#"reg973689048B04F03AA9C98C5A1F6AE6D3"#),
            Value::from(1),
            Value::from(r#"Software\Airshipper"#),
            Value::from(r#"installed"#),
            Value::from(r#"#1"#),
            Value::from(r#"ApplicationShortcut"#),
        ])
        .row(vec![
            Value::from(r#"reg65FB3BC78BA56162E6CFE88513453D5D"#),
            Value::from(1),
            Value::from(r#"Software\Airshipper"#),
            Value::from(r#"DesktopShortcut"#),
            Value::from(r#"#1"#),
            Value::from(r#"ApplicationShortcutDesktop"#),
        ])
        .row(vec![
            Value::from(r#"regB8E49E39311754CE2F424A110DF17A1B"#),
            Value::from(1),
            Value::from(r#"Software\Airshipper"#),
            Value::from(r#"DesktopCompatibilityShortcut"#),
            Value::from(r#"#1"#),
            Value::from(r#"ApplicationShortcutCompatibilityDesktop"#),
        ])
        .row(vec![
            Value::from(r#"regCDE3646517A7994C3BA6B36EAFF468D2"#),
            Value::from(1),
            Value::from(r#"Software\Airshipper"#),
            Value::from(r#"InstallationPath"#),
            Value::from(r#"[APPLICATIONFOLDER]"#),
            Value::from(r#"ApplicationShortcut"#),
        ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(REMOVE_FILE, vec![
            Column::build("FileKey")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Component_")
                .category(Category::Identifier)
                .string(72),
            Column::build("FileName")
                .nullable()
                .localizable()
                .category(Category::WildCardFilename)
                .string(255),
            Column::build("DirProperty")
                .category(Category::Identifier)
                .string(72),
            Column::build("InstallMode")
                .enum_values(&["1", "2", "3"])
                .int16(),
        ])
        .unwrap();

    let query = Insert::into(REMOVE_FILE)
        .row(vec![
            Value::from("CleanUpShortCut"),
            Value::from("ApplicationShortcut"),
            Value::Null,
            Value::from("ApplicationProgramsFolder"),
            Value::from(2),
        ])
        .row(vec![
            Value::from("CleanUpDesktopShortcut"),
            Value::from("ApplicationShortcutDesktop"),
            Value::Null,
            Value::from("DesktopFolder"),
            Value::from(2),
        ])
        .row(vec![
            Value::from("CleanUpDesktopCompatibilityShortcut"),
            Value::from("ApplicationShortcutCompatibilityDesktop"),
            Value::Null,
            Value::from("DesktopFolder"),
            Value::from(2),
        ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(SHORTCUT, vec![
            Column::build("Shortcut")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Directory_")
                .category(Category::Identifier)
                .string(72),
            Column::build("Name")
                .localizable()
                .category(Category::Filename)
                .string(128),
            Column::build("Component_")
                .category(Category::Identifier)
                .string(72),
            Column::build("Target")
                .category(Category::Shortcut)
                .string(72),
            Column::build("Arguments")
                .nullable()
                .category(Category::Formatted)
                .string(255),
            Column::build("Description")
                .nullable()
                .localizable()
                .category(Category::Text)
                .string(255),
            Column::build("Hotkey").nullable().range(0, 32767).int16(),
            Column::build("Icon_")
                .nullable()
                .category(Category::Identifier)
                .string(72),
            Column::build("IconIndex")
                .nullable()
                .range(-32767, 32767)
                .int16(),
            Column::build("ShowCmd")
                .nullable()
                .enum_values(&["1", "3", "7"])
                .int16(),
            Column::build("WkDir")
                .nullable()
                .category(Category::Identifier)
                .string(72),
            Column::build("DisplayResourceDLL")
                .nullable()
                .category(Category::Formatted)
                .string(255),
            Column::build("DisplayResourceId")
                .nullable()
                .range(0, 32767)
                .int16(),
            Column::build("DescriptionResourceDLL")
                .nullable()
                .category(Category::Formatted)
                .string(255),
            Column::build("DescriptionResourceId")
                .nullable()
                .range(0, 32767)
                .int16(),
        ])
        .unwrap();

    let query = Insert::into(SHORTCUT)
        .row(vec![
            Value::from("ApplicationStartMenuShortcut"),
            Value::from("ApplicationProgramsFolder"),
            Value::from("yfcwg8kc|Airshipper"),
            Value::from("ApplicationShortcut"),
            Value::from("[!airshipper.exe]"),
            Value::Null,
            Value::from("Airshipper"),
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
            Value::from("APPLICATIONFOLDER"),
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
        ])
        .row(vec![
            Value::from("ApplicationStartMenuCompatibilityShortcut"),
            Value::from("ApplicationProgramsFolder"),
            Value::from("al-f3ru1|Airshipper (compatibility mode)"),
            Value::from("ApplicationShortcut"),
            Value::from("[!airshipper.exe]"),
            Value::from("run"),
            Value::from(
                "Will update and start Veloren on all kinds of hardware even if \
                 Airshipper itself wouldn't be supported.",
            ),
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
            Value::from("APPLICATIONFOLDER"),
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
        ])
        .row(vec![
            Value::from("ApplicationDesktopShortcut"),
            Value::from("DesktopFolder"),
            Value::from("ps7cbuld|Airshipper"),
            Value::from("ApplicationShortcutDesktop"),
            Value::from("[!airshipper.exe]"),
            Value::Null,
            Value::from("Provides automatic updates for the voxel RPG Veloren."),
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
            Value::from("APPLICATIONFOLDER"),
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
        ])
        .row(vec![
            Value::from("ApplicationDesktopCompatibilityShortcut"),
            Value::from("DesktopFolder"),
            Value::from("a4hs0xx2|Airshipper (compatibility mode)"),
            Value::from("ApplicationShortcutCompatibilityDesktop"),
            Value::from("[!airshipper.exe]"),
            Value::from("run"),
            Value::from(
                "Will update and start Veloren on all kinds of hardware even if \
                 Airshipper itself wouldn't be supported.",
            ),
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
            Value::from("APPLICATIONFOLDER"),
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
        ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(SIGNATURE, vec![
            Column::build("Signature")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("FileName")
                .category(Category::Text)
                .string(255),
            Column::build("MinVersion")
                .nullable()
                .category(Category::Text)
                .string(20),
            Column::build("MaxVersion")
                .nullable()
                .category(Category::Text)
                .string(20),
            Column::build("MinSize")
                .nullable()
                .range(0, 2147483647)
                .int32(),
            Column::build("MaxSize")
                .nullable()
                .range(0, 2147483647)
                .int32(),
            Column::build("MinDate")
                .nullable()
                .range(0, 2147483647)
                .int32(),
            Column::build("MaxDate")
                .nullable()
                .range(0, 2147483647)
                .int32(),
            Column::build("Languages")
                .nullable()
                .category(Category::Language)
                .string(255),
        ])
        .unwrap();

    // empty

    package
        .create_table(TEXT_STYLE, vec![
            Column::build("TextStyle")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("FaceName")
                .category(Category::Text)
                .string(32),
            Column::build("Size").range(0, 32767).int16(),
            Column::build("Color").nullable().range(0, 16777215).int32(),
            Column::build("StyleBits").nullable().range(0, 15).int16(),
        ])
        .unwrap();

    let query = Insert::into(TEXT_STYLE)
        .row(vec![
            Value::from("WixUI_Font_Normal"),
            Value::from("Tahoma"),
            Value::from(8),
            Value::Null,
            Value::Null,
        ])
        .row(vec![
            Value::from("WixUI_Font_Bigger"),
            Value::from("Tahoma"),
            Value::from(12),
            Value::Null,
            Value::Null,
        ])
        .row(vec![
            Value::from("WixUI_Font_Title"),
            Value::from("Tahoma"),
            Value::from(9),
            Value::Null,
            Value::from(1),
        ]);
    package.insert_rows(query).unwrap();

    package
        .create_table(UI_TEXT, vec![
            Column::build("Key")
                .primary_key()
                .category(Category::Identifier)
                .string(72),
            Column::build("Text")
                .nullable()
                .localizable()
                .category(Category::Text)
                .string(255),
        ])
        .unwrap();

    let query = Insert::into(UI_TEXT)
        .row(vec![
            Value::from("NewFolder"),
            Value::from("Folder|New Folder"),
        ])
        .row(vec![Value::from("AbsentPath"), Value::Null])
        .row(vec![Value::from("bytes"), Value::from("bytes")])
        .row(vec![Value::from("GB"), Value::from("GB")])
        .row(vec![Value::from("KB"), Value::from("KB")])
        .row(vec![Value::from("MB"), Value::from("MB")])
        .row(vec![
            Value::from("MenuAbsent"),
            Value::from("Entire feature will be unavailable"),
        ])
        .row(vec![
            Value::from("MenuAdvertise"),
            Value::from("Feature will be installed when required"),
        ])
        .row(vec![
            Value::from("MenuAllCD"),
            Value::from("Entire feature will be installed to run from CD"),
        ])
        .row(vec![
            Value::from("MenuAllLocal"),
            Value::from("Entire feature will be installed on local hard drive"),
        ])
        .row(vec![
            Value::from("MenuAllNetwork"),
            Value::from("Entire feature will be installed to run from network"),
        ])
        .row(vec![
            Value::from("MenuCD"),
            Value::from("Will be installed to run from CD"),
        ])
        .row(vec![
            Value::from("MenuLocal"),
            Value::from("Will be installed on local hard drive"),
        ])
        .row(vec![
            Value::from("MenuNetwork"),
            Value::from("Will be installed to run from network"),
        ])
        .row(vec![
            Value::from("ScriptInProgress"),
            Value::from("Gathering required information..."),
        ])
        .row(vec![
            Value::from("SelAbsentAbsent"),
            Value::from("This feature will remain uninstalled"),
        ])
        .row(vec![
            Value::from("SelAbsentAdvertise"),
            Value::from("This feature will be set to be installed when required"),
        ])
        .row(vec![
            Value::from("SelAbsentCD"),
            Value::from("This feature will be installed to run from CD"),
        ])
        .row(vec![
            Value::from("SelAbsentLocal"),
            Value::from("This feature will be installed on the local hard drive"),
        ])
        .row(vec![
            Value::from("SelAbsentNetwork"),
            Value::from("This feature will be installed to run from the network"),
        ])
        .row(vec![
            Value::from("SelAdvertiseAbsent"),
            Value::from("This feature will become unavailable"),
        ])
        .row(vec![
            Value::from("SelAdvertiseAdvertise"),
            Value::from("Will be installed when required"),
        ])
        .row(vec![
            Value::from("SelAdvertiseCD"),
            Value::from("This feature will be available to run from CD"),
        ])
        .row(vec![
            Value::from("SelAdvertiseLocal"),
            Value::from("This feature will be installed on your local hard drive"),
        ])
        .row(vec![
            Value::from("SelAdvertiseNetwork"),
            Value::from("This feature will be available to run from the network"),
        ])
        .row(vec![
            Value::from("SelCDAbsent"),
            Value::from(
                "This feature will be uninstalled completely, you won't be able to run \
                 it from CD",
            ),
        ])
        .row(vec![
            Value::from("SelCDAdvertise"),
            Value::from(
                "This feature will change from run from CD state to set to be installed \
                 when required",
            ),
        ])
        .row(vec![
            Value::from("SelCDCD"),
            Value::from("This feature will remain to be run from CD"),
        ])
        .row(vec![
            Value::from("SelCDLocal"),
            Value::from(
                "This feature will change from run from CD state to be installed on the \
                 local hard drive",
            ),
        ])
        .row(vec![
            Value::from("SelChildCostNeg"),
            Value::from("This feature frees up [1] on your hard drive."),
        ])
        .row(vec![
            Value::from("SelChildCostPos"),
            Value::from("This feature requires [1] on your hard drive."),
        ])
        .row(vec![
            Value::from("SelCostPending"),
            Value::from("Compiling cost for this feature..."),
        ])
        .row(vec![
            Value::from("SelLocalAbsent"),
            Value::from("This feature will be completely removed"),
        ])
        .row(vec![
            Value::from("SelLocalAdvertise"),
            Value::from(
                "This feature will be removed from your local hard drive, but will be \
                 set to be installed when required",
            ),
        ])
        .row(vec![
            Value::from("SelLocalCD"),
            Value::from(
                "This feature will be removed from your local hard drive, but will be \
                 still available to run from CD",
            ),
        ])
        .row(vec![
            Value::from("SelLocalLocal"),
            Value::from("This feature will remain on your local hard drive"),
        ])
        .row(vec![
            Value::from("SelLocalNetwork"),
            Value::from(
                "This feature will be removed from your local hard drive, but will be \
                 still available to run from the network",
            ),
        ])
        .row(vec![
            Value::from("SelNetworkAbsent"),
            Value::from(
                "This feature will be uninstalled completely, you won't be able to run \
                 it from the network",
            ),
        ])
        .row(vec![
            Value::from("SelNetworkAdvertise"),
            Value::from(
                "This feature will change from run from network state to set to be \
                 installed when required",
            ),
        ])
        .row(vec![
            Value::from("SelNetworkLocal"),
            Value::from(
                "This feature will change from run from network state to be installed \
                 on the local hard drive",
            ),
        ])
        .row(vec![
            Value::from("SelNetworkNetwork"),
            Value::from("This feature will remain to be run from the network"),
        ])
        .row(vec![
            Value::from("SelParentCostNegNeg"),
            Value::from(
                "This feature frees up [1] on your hard drive. It has [2] of [3] \
                 subfeatures selected. The subfeatures free up [4] on your hard drive.",
            ),
        ])
        .row(vec![
            Value::from("SelParentCostNegPos"),
            Value::from(
                "This feature frees up [1] on your hard drive. It has [2] of [3] \
                 subfeatures selected. The subfeatures require [4] on your hard drive.",
            ),
        ])
        .row(vec![
            Value::from("SelParentCostPosNeg"),
            Value::from(
                "This feature requires [1] on your hard drive. It has [2] of [3] \
                 subfeatures selected. The subfeatures free up [4] on your hard drive.",
            ),
        ])
        .row(vec![
            Value::from("SelParentCostPosPos"),
            Value::from(
                "This feature requires [1] on your hard drive. It has [2] of [3] \
                 subfeatures selected. The subfeatures require [4] on your hard drive.",
            ),
        ])
        .row(vec![
            Value::from("TimeRemaining"),
            Value::from("Time remaining:{ [1] minutes}{ [2] seconds}"),
        ])
        .row(vec![
            Value::from("VolumeCostAvailable"),
            Value::from("Available"),
        ])
        .row(vec![
            Value::from("VolumeCostDifference"),
            Value::from("Difference"),
        ])
        .row(vec![
            Value::from("VolumeCostRequired"),
            Value::from("Required"),
        ])
        .row(vec![
            Value::from("VolumeCostSize"),
            Value::from("Disk Size"),
        ])
        .row(vec![Value::from("VolumeCostVolume"), Value::from("Volume")]);
    package.insert_rows(query).unwrap();

    package
        .create_table(UPGRADE, vec![
            Column::build("UpgradeCode")
                .primary_key()
                .category(Category::Guid)
                .string(38),
            Column::build("VersionMin")
                .primary_key()
                .nullable()
                .category(Category::Text)
                .string(20),
            Column::build("VersionMax")
                .primary_key()
                .nullable()
                .category(Category::Text)
                .string(20),
            Column::build("Language")
                .primary_key()
                .nullable()
                .category(Category::Language)
                .string(255),
            Column::build("Attributes")
                .primary_key()
                .range(0, 2147483647)
                .int32(),
            Column::build("Remove")
                .nullable()
                .category(Category::Formatted)
                .string(255),
            Column::build("ActionProperty")
                .category(Category::UpperCase)
                .string(72),
        ])
        .unwrap();

    let query = Insert::into(UPGRADE)
        .row(vec![
            Value::from("{1715788C-2FC7-44D7-912D-2B46202C2FD9}"),
            Value::Null,
            Value::from(version),
            Value::Null,
            Value::from(1),
            Value::Null,
            Value::from("WIX_UPGRADE_DETECTED"),
        ])
        .row(vec![
            Value::from("{1715788C-2FC7-44D7-912D-2B46202C2FD9}"),
            Value::from(version),
            Value::Null,
            Value::Null,
            Value::from(2),
            Value::Null,
            Value::from("WIX_DOWNGRADE_DETECTED"),
        ]);
    package.insert_rows(query).unwrap();

    let icon = include_bytes!("./Veloren.ico");
    let mut icon1 = package.write_stream("Icon.ProductICO").unwrap();
    icon1.write_all(icon).unwrap();
    let mut icon2 = package.write_stream("Binary.WixUI_Ico_Exclam").unwrap();
    icon2.write_all(icon).unwrap();
    let mut icon3 = package.write_stream("Binary.WixUI_Ico_Info").unwrap();
    icon3.write_all(icon).unwrap();

    let dialog = include_bytes!("./Dialog.bmp");
    let mut dialog1 = package.write_stream("Binary.WixUI_Bmp_Dialog").unwrap();
    dialog1.write_all(dialog).unwrap();

    let banner = include_bytes!("./Banner.bmp");
    let mut banner1 = package.write_stream("Binary.WixUI_Bmp_Banner").unwrap();
    banner1.write_all(banner).unwrap();

    let mut binary2 = package.write_stream("Binary.WixCA").unwrap();
    binary2.write_all(include_bytes!("./Binary.WixCA")).unwrap();

    let mut binary3 = package.write_stream("Binary.WixUI_Bmp_New").unwrap();
    binary3
        .write_all(include_bytes!("./Binary.WixUI_Bmp_New"))
        .unwrap();

    let mut binary4 = package.write_stream("Binary.WixUI_Bmp_Up").unwrap();
    binary4
        .write_all(include_bytes!("./Binary.WixUI_Bmp_Up"))
        .unwrap();


    let mut airshipper = package.write_stream("media1.cab").unwrap();
        airshipper
            .write_all(&cab_file(airshipper_exe))
            .unwrap();



    /*
    package
        .create_table("_Columns", vec![
            Column::build("Table").primary_key().string(64),
            Column::build("Number").primary_key().int16(),
            Column::build("Name").string(64),
            Column::build("Type").int16(),
        ])
        .unwrap();

    package
        .create_table("_Tables", vec![
            Column::build("Name").primary_key().string(64),
        ])
        .unwrap();

    package
        .create_table("_Validation", vec![
            Column::build("Table")
                .primary_key()
                .category(Category::Identifier)
                .string(32),
            Column::build("Column")
                .primary_key()
                .category(Category::Identifier)
                .string(32),
            Column::build("Nullable").enum_values(&["Y", "N"]).string(4),
            Column::build("MinValue")
                .nullable()
                .range(-2147483647, 2147483647)
                .int32(),
            Column::build("MaxValue")
                .nullable()
                .range(-2147483647, 2147483647)
                .int32(),
            Column::build("KeyTable")
                .nullable()
                .category(Category::Identifier)
                .string(255),
            Column::build("KeyColumn").nullable().range(1, 32).int16(),
            Column::build("Category")
                .nullable()
                .enum_values(&[
                    "Text",
                    "Formatted",
                    "Template",
                    "Condition",
                    "Guid",
                    "Path",
                    "Version",
                    "Language",
                    "Identifier",
                    "Binary",
                    "UpperCase",
                    "LowerCase",
                    "Filename",
                    "Paths",
                    "AnyPath",
                    "WildCardFilename",
                    "RegPath",
                    "CustomSource",
                    "Property",
                    "Cabinet",
                    "Shortcut",
                    "FormattedSDDLText",
                    "Integer",
                    "DoubleInteger",
                    "TimeDate",
                    "DefaultDir",
                ])
                .string(32),
            Column::build("Set")
                .nullable()
                .category(Category::Text)
                .string(255),
            Column::build("Description")
                .nullable()
                .category(Category::Text)
                .string(255),
        ])
        .unwrap(); */

    package
}

// put file into a cab
fn cab_file(airshipper_exe: &[u8]) -> Vec<u8> {
    use zip::write::SimpleFileOptions;
    let buf = vec![];
    let mut zip = ZipWriter::new(std::io::Cursor::new(buf));
    let options =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip.add_directory("media1", options).unwrap();
    zip
        .start_file_from_path(Path::new("media1/airshipper.exe"), options)
        .unwrap();
    zip.write_all(airshipper_exe).unwrap();
    zip.finish().unwrap().into_inner()
}

#[allow(dead_code)]
fn display_file(
    bytes: &[u8],
) -> Result<Package<Cursor<Vec<u8>>>, Box<dyn std::error::Error>> {
    let buf = Vec::from(bytes);

    let buff = Cursor::new(buf);

    let mut package = Package::open(buff).unwrap();

    for table in package.tables() {
        println!("Table: {}", table.name());
        let keys = table.primary_key_indices();
        let mut columns = Vec::new();
        for (i, column) in table.columns().into_iter().enumerate() {
            let mut s = format!(r#"Column::build("{}")"#, column.name());
            if keys.contains(&i) {
                s += ".primary_key()";
            }
            if column.is_nullable() {
                s += ".nullable()";
            }
            if column.is_localizable() {
                s += ".localizable()";
            }
            if let Some((min, max)) = column.value_range() {
                s += &format!(".range({min}, {max})");
            }
            if let Some(e) = column.enum_values() {
                s += &format!(".enum_values(&{:?})", e);
            }
            if let Some(ct) = column.category() {
                s += &format!(".category(Category::{ct})");
            }
            s += &format!(".{}", column.coltype());

            columns.push(s);
        }
        let inner = columns.join(", ");
        println!(
            r#"package.create_table("{}", vec![{}]).unwrap();"#,
            table.name(),
            inner
        );

        println!("");
    }

    let tables: Vec<String> = package.tables().map(|t| t.name().to_string()).collect();
    for table in tables {
        let rows = package.select_rows(msi::Select::table(&table)).unwrap();
        let mut all_data = Vec::new();
        for r in rows {
            let l = r.len();
            let mut row = Vec::new();
            for i in 0..l {
                match &r[i] {
                    Value::Null => row.push("Value::Null".to_string()),
                    Value::Str(v) => row.push(format!(r##"Value::from(r#"{}"#)"##, v)),
                    Value::Int(v) => row.push(format!(r##"Value::from({})"##, v)),
                }
            }
            let inner = row.join(", ");
            all_data.push(format!(".row(vec![{inner}])"));
        }
        let outer = all_data.join("");
        println!("--{}", &table);
        println!(
            "let query = Insert::into(\"{}\"){}; \n package.insert_rows(query).unwrap();",
            table, outer
        );
        println!("");
    }

    Ok(package)
}

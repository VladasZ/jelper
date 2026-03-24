[Setup]
AppName=Jelper
AppVersion={#AppVersion}
AppPublisher=Advantage Group
DefaultDirName={localappdata}\Programs\Jelper
DefaultGroupName=Jelper
DisableProgramGroupPage=no
OutputBaseFilename=jelper-installer
SetupIconFile=jelper.ico
Compression=lzma
SolidCompression=yes
PrivilegesRequired=lowest
ChangesEnvironment=yes

[Files]
Source: "dist\jelper-windows.exe"; DestDir: "{app}"; DestName: "jelper.exe"

[Tasks]
Name: "startmenu"; Description: "Create a Start Menu shortcut"; GroupDescription: "Shortcuts:"
Name: "desktopicon"; Description: "Create a Desktop shortcut"; GroupDescription: "Shortcuts:"

[Icons]
Name: "{group}\Jelper"; Filename: "{app}\jelper.exe"; IconFilename: "{app}\jelper.exe"; Tasks: startmenu
Name: "{commondesktop}\Jelper"; Filename: "{app}\jelper.exe"; IconFilename: "{app}\jelper.exe"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; \
  ValueData: "{olddata};{app}"; \
  Check: PathNotAdded('{app}')

[Code]
function PathNotAdded(NewPath: string): Boolean;
var
  CurrentPath: string;
begin
  if RegQueryStringValue(HKCU, 'Environment', 'Path', CurrentPath) then
    Result := Pos(LowerCase(NewPath), LowerCase(CurrentPath)) = 0
  else
    Result := True;
end;

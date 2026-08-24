use serde::Serialize;
use std::process::Command;

#[derive(Serialize)]
struct Gpu {
    vendor: String,
    pci_id: String,
    legacy_390: Option<String>,
    legacy_470: Option<String>,
}

#[derive(Serialize)]
struct GpuReport {
    gpus: Vec<Gpu>,
}

fn main() {
    let output = Command::new("lspci")
        .arg("-nn")
        .output()
        .expect("Failed to run lspci. Make sure pciutils is installed!");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut gpus = Vec::new();

    for line in stdout.lines() {
        let isgpu = line.contains("VGA compatible controller")
            || line.contains("3D controller")
            || line.contains("Display controller");

        let mut warning_470 = None;
        let mut warning_390 = None;
        let legacy_390 = ["10de:0c", "10de:0d", "10de:0e", "10de:0f", "10de:10"];

        let legacy_470 = [
            "10de:11", "10de:12", "10de:13", "10de:14", "10de:15", "10de:16", "10de:17", "10de:18",
            "10de:19", "10de:1a", "10de:1b", "10de:1c", "10de:1d", "10de:1e", "10de:1f",
        ];

        if !isgpu {
            continue;
        }

        if let Some(start_index) = line.rfind('[') {
            let idblock = &line[start_index + 1..];

            let fullid = idblock.split(']').next().unwrap();
            let vendorid = fullid.split(":").next().unwrap();

            let vendorname = match vendorid {
                "10de" => "nvidia",
                "1002" => "amd",
                "8086" => "intel",
                _ => continue,
            };

            let isduplicated = gpus.iter().any(|gpu: &Gpu| gpu.vendor == vendorname);
            let islegacy_390 = legacy_390.iter().any(|prefix| fullid.starts_with(prefix));
            let islegacy_470 = legacy_470.iter().any(|prefix| fullid.starts_with(prefix));

            if !isduplicated {
                if islegacy_470 {
                    warning_470 = Some(
                            "Legacy GPU detected please run sudo pacman -S nvidia-470xx-dkms in the Terminal".to_string()
                        )
                }
                if islegacy_390 {
                    warning_390 = Some("Legacy GPU detected please run sudo pacman -S nvidia-390xx-dkms in the Terminal".to_string())
                }
                gpus.push(Gpu {
                    vendor: vendorname.to_string(),
                    pci_id: fullid.to_string(),
                    legacy_390: warning_390,
                    legacy_470: warning_470,
                });
            }
        }
    }

    let report = GpuReport { gpus };
    let json_output = serde_json::to_string(&report).unwrap();
    println!("{}", json_output);
}

use std::collections::HashMap;

use raw_cpuid::CpuId;
use serde::Deserialize;
use wmi::{COMLibrary, Variant, WMIConnection};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cpuid = CpuId::new();

    if let Some(vf) = cpuid.get_vendor_info() {
        println!("CPU Vendor: {}", vf.as_str());
    }

    if let Some(fi) = cpuid.get_feature_info() {
        println!(
            "Family: {}, Model: {}, Stepping: {}",
            fi.family_id(),
            fi.model_id(),
            fi.stepping_id(),
        );

        println!("--- CPU Feature Flags ---");
        println!("- SSE3: {}", yes_no(fi.has_sse3()));
        println!("- PCLMULQDQ: {}", yes_no(fi.has_pclmulqdq()));
        println!("- MONITOR/MWAIT: {}", yes_no(fi.has_monitor_mwait()));
        println!("- CPL Qualified Debug Store: {}", yes_no(fi.has_cpl()));
        println!(
            "- Virtual Machine Extensions (VMX): {}",
            yes_no(fi.has_vmx())
        );
        println!("- Safer Mode Extensions (SMX): {}", yes_no(fi.has_smx()));
        println!("- Enhanced Intel SpeedStep: {}", yes_no(fi.has_eist()));
        println!("- Thermal Monitor 2: {}", yes_no(fi.has_tm2()));
        println!("- SSSE3: {}", yes_no(fi.has_ssse3()));
        println!("- L1 Context ID: {}", yes_no(fi.has_cnxtid()));
        println!("- FMA: {}", yes_no(fi.has_fma()));
        println!("- CMPXCHG16B: {}", yes_no(fi.has_cmpxchg16b()));
        println!("- PDCM: {}", yes_no(fi.has_pdcm()));
        println!("- PCID: {}", yes_no(fi.has_pcid()));
        println!("- DCA: {}", yes_no(fi.has_dca()));
        println!("- SSE4.1: {}", yes_no(fi.has_sse41()));
        println!("- SSE4.2: {}", yes_no(fi.has_sse42()));
        println!("- x2APIC: {}", yes_no(fi.has_x2apic()));
        println!("- MOVBE: {}", yes_no(fi.has_movbe()));
        println!("- POPCNT: {}", yes_no(fi.has_popcnt()));
        println!("- TSC Deadline: {}", yes_no(fi.has_tsc_deadline()));
        println!("- AES-NI: {}", yes_no(fi.has_aesni()));
        println!("- XSAVE/XRSTOR: {}", yes_no(fi.has_xsave()));
        println!("- OSXSAVE Enabled: {}", yes_no(fi.has_oxsave()));
        println!("- AVX: {}", yes_no(fi.has_avx()));
        println!("- F16C: {}", yes_no(fi.has_f16c()));
        println!("- RDRAND: {}", yes_no(fi.has_rdrand()));
        println!("- Hypervisor: {}", yes_no(fi.has_hypervisor()));

        println!();
        println!("--- Legacy Feature Flags (EDX) ---");
        println!("- FPU: {}", yes_no(fi.has_fpu()));
        println!("- VME: {}", yes_no(fi.has_vme()));
        println!("- DE: {}", yes_no(fi.has_de()));
        println!("- PSE: {}", yes_no(fi.has_pse()));
        println!("- TSC: {}", yes_no(fi.has_tsc()));
        println!("- MSR: {}", yes_no(fi.has_msr()));
        println!("- PAE: {}", yes_no(fi.has_pae()));
        println!("- MCE: {}", yes_no(fi.has_mce()));
        println!("- CMPXCHG8B: {}", yes_no(fi.has_cmpxchg8b()));
        println!("- APIC: {}", yes_no(fi.has_apic()));
        println!("- SYSENTER/SYSEXIT: {}", yes_no(fi.has_sysenter_sysexit()));
        println!("- MTRR: {}", yes_no(fi.has_mtrr()));
        println!("- PGE: {}", yes_no(fi.has_pge()));
        println!("- MCA: {}", yes_no(fi.has_mca()));
        println!("- CMOV: {}", yes_no(fi.has_cmov()));
        println!("- PAT: {}", yes_no(fi.has_pat()));
        println!("- PSE36: {}", yes_no(fi.has_pse36()));
        println!("- PSN: {}", yes_no(fi.has_psn()));
        println!("- CLFLUSH: {}", yes_no(fi.has_clflush()));
        println!("- Debug Store (DS): {}", yes_no(fi.has_ds()));
        println!("- ACPI: {}", yes_no(fi.has_acpi()));
        println!("- MMX: {}", yes_no(fi.has_mmx()));
        println!("- FXSAVE/FXRSTOR: {}", yes_no(fi.has_fxsave_fxstor()));
        println!("- SSE: {}", yes_no(fi.has_sse()));
        println!("- SSE2: {}", yes_no(fi.has_sse2()));
        println!("- Self Snoop");
        println!("- Hyper-Threading (HTT): {}", yes_no(fi.has_htt()));
        println!("- Thermal Monitor: {}", yes_no(fi.has_tm()));
        println!("- Pending Break Enable (PBE): {}", yes_no(fi.has_pbe()));
    } else {
        println!("No feature information available");
    }

    if let Some(ef) = cpuid.get_extended_feature_info() {
        println!("- AVX2: {}", yes_no(ef.has_avx2()));
        println!("- AVX-512F: {}", yes_no(ef.has_avx512f()));
        println!("- SHA: {}", yes_no(ef.has_sha()));
    }

    if let Some(cparams) = cpuid.get_cache_parameters() {
        for cache in cparams {
            let size = cache.associativity()
                * cache.physical_line_partitions()
                * cache.coherency_line_size()
                * cache.sets();
            println!("L{}-Cache size: {} bytes", cache.level(), size);
        }
    } else {
        println!("No cache parameter information available");
    }

    println!();
    println!("--- Sensors (via LibreHardwareMonitor) ---");

    let com = COMLibrary::new()?;
    let wmi = WMIConnection::with_namespace_path("root\\LibreHardwareMonitor", com.into())?;

    let results: Vec<HashMap<String, Variant>> =
        wmi.raw_query("SELECT Identifier, Name, SensorType, Value, Min, Max FROM Sensor")?;

    if results.is_empty() {
        println!("(no sensors found — run LibreHardwareMonitor as Administrator)");
        return Ok(());
    }

    for row in results {
        let id = extract_string(row.get("Identifier"));
        let name = extract_string(row.get("Name"));
        let sensor_type = extract_string(row.get("SensorType"));
        let val = extract_f32(row.get("Value"));
        let min = extract_f32(row.get("Min"));
        let max = extract_f32(row.get("Max"));

        println!(
            "{:<27} | {:<12} | Cur: {:>10.2} | Min: {:>10.2} | Max: {:>10.2} | {}",
            name, sensor_type, val, min, max, id
        );
    }

    Ok(())
}

fn extract_string(v: Option<&Variant>) -> String {
    match v {
        Some(Variant::String(s)) => s.clone(),
        Some(Variant::Bool(b)) => b.to_string(),
        Some(Variant::I4(i)) => i.to_string(),
        Some(Variant::UI4(u)) => u.to_string(),
        Some(Variant::R4(f)) => format!("{f}"),
        Some(Variant::R8(f)) => format!("{f}"),
        Some(Variant::Null) | None => String::new(),
        Some(x) => format!("{:?}", x),
    }
}

fn extract_f32(v: Option<&Variant>) -> f32 {
    match v {
        Some(Variant::R4(f)) => *f,
        Some(Variant::R8(f)) => *f as f32,
        Some(Variant::I4(i)) => *i as f32,
        Some(Variant::UI4(u)) => *u as f32,
        Some(Variant::String(s)) => s.parse::<f32>().unwrap_or_default(),
        _ => 0.0,
    }
}

fn yes_no(val: bool) -> &'static str {
    if val { "Yes" } else { "No" }
}

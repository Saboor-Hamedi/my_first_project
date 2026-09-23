# Web Security Scanner (`webscan`)

MindForge includes a built-in security auditing and vulnerability scanning engine designed to inspect web targets directly from your workspace.

---

## Quick Start & Command Syntax

Launch a scan from the command bar (:CMD) at any time:

```text
:scan <url> [--full] [--forms] [--delay <ms>] [--timeout <secs>] [--note "<text>"]
```

MindForge runs the scan asynchronously in a background thread so your editor and writing workflow remain completely uninterrupted. Once finished, MindForge automatically opens the interactive **Scan Report** dashboard.

---

## Command Parameters & Flags

| Parameter | Type | Default | Description |
|:---|:---|:---|:---|
| `<url>` | Positional String | *Required* | Target domain or URL (e.g., `https://example.com` or `example.com`). If scheme is omitted, `https://` is automatically prepended. |
| `--full` | Flag (boolean) | `false` | Enables deep active checks: sensitive file / path disclosure (`.env`, `.git`, backups, admin endpoints) and form injection tests. |
| `--forms` / `--probe-forms` | Flag (boolean) | `false` | Enables active injection probes against discovered HTML forms (XSS, SQLi, open redirects). **Requires `--full`**. |
| `--delay <ms>` | Integer (u64) | `200` | Throttle delay in milliseconds applied before **every** outbound HTTP request to avoid triggering rate limiters or overloading targets. |
| `--timeout <secs>` | Integer (u64) | `10` | Maximum network timeout in seconds for each HTTP request. |
| `--note "<text>"` | Quoted String | `None` | Custom audit note, client name, or ticket reference saved alongside the report in your SQLite database. |

---

## Command Examples

### 1. Fast Passive Scan (Default)
Inspects security headers, cookie flags, TLS certificates, and server fingerprinting without sending intrusive probes:
```text
:scan example.com
```

### 2. Full Active Audit with Custom Note
Enables path disclosure scans and tags the scan report for client deliverables:
```text
:scan https://staging.myapp.dev --full --note "Pre-launch security review"
```

### 3. Deep Form & Injection Testing
Enables both path disclosure and active form input probing:
```text
:scan https://testphp.vulnweb.com --full --forms --note "OWASP Top 10 Form Audit"
```

### 4. Throttled Low-Impact Scan
Adds a 500ms delay between requests with a 15s timeout to remain stealthy and avoid WAF/rate-limit bans:
```text
:scan api.target.com --full --delay 500 --timeout 15 --note "API rate-limited scan"
```

---

## Programmatic Rust API (`ScanOptions`)

For headless automation or Rust scripting, configure the scan through `webscan::ScanOptions`:

```rust
use webscan::{scan, ScanOptions};

let options = ScanOptions {
    full: true,               // Enable active path disclosure + injection
    probe_forms: true,        // Enable form injection specifically (requires full: true)
    delay_ms: 250,            // 250ms throttle between outbound requests
    timeout_secs: 12,         // 12-second HTTP timeout
    note: Some("CI Pipeline Security Gate".to_string()),
};

let result = scan("https://example.com", &options)?;
println!("Score: {}% (Grade {})", result.score(), result.grade());
for finding in &result.findings {
    println!("[{:?}] [{}] {}", finding.category, finding.severity.as_str(), finding.title);
}
```

---

## Scan Capabilities & Checks

The scanner runs a comprehensive multi-point audit across several key security vectors:

### 1. HTTP Security Headers (Passive)
Analyzes whether recommended defensive headers are configured:
- **Strict-Transport-Security (HSTS)**: Enforces encrypted HTTPS connections.
- **Content-Security-Policy (CSP)**: Mitigates Cross-Site Scripting (XSS) and data injection attacks.
- **X-Frame-Options**: Prevents clickjacking and UI redressing.
- **X-Content-Type-Options**: Blocks MIME-type sniffing (`nosniff`).
- **Referrer-Policy**: Protects sensitive URL data in referral paths.
- **Permissions-Policy**: Restricts browser features (camera, microphone, geolocation).

### 2. Cookie Security Flags (Passive)
- Checks all `Set-Cookie` directives for `Secure`, `HttpOnly`, and `SameSite` flags.

### 3. SSL/TLS Certificate Analysis (Passive)
- Verifies certificate validity, cipher suite strength, and expiration date.
- Warns if certificates are near expiration or self-signed.

### 4. Information Disclosure & Sensitive Paths (Active with `--full`)
- Probes for exposed source control (`.git/HEAD`), environment variables (`.env`), backup files (`config.php.bak`, `dump.sql`), and unauthenticated administrative panels.

### 5. Input Form & Injection Probing (Active with `--full --forms`)
- Discovers HTML `<form>` inputs and evaluates vulnerability to XSS reflections, SQL error disclosures, and insecure endpoints.

---

## Scoring & Risk Grading

Each scan evaluates vulnerabilities and produces an aggregate score:
- **A+ / A (90–100%)**: Excellent posture. All major headers and encryption safeguards in place.
- **B (80–89%)**: Good posture with minor configuration improvements recommended.
- **C (70–79%)**: Moderate risk. Multiple standard headers missing.
- **D (60–69%)**: High risk. Weak encryption or critical missing defenses.
- **F (< 60%)**: Critical vulnerabilities detected or insecure transmission.

---

## Scan History (`:scans`)

To review previous audit reports:
- Open the command bar and enter `:scans`.
- Alternatively, click the **Scans** icon in the sidebar.
- Navigate previous scans with `j`/`k` or arrow keys.
- Press `Enter` to open any historical report.

---

## Exporting Reports to Notes

You can convert any scan report into an editable Markdown note in your library:
- While viewing a report, click the **Export to Note** button or type `:export`.
- MindForge creates a structured note containing the target summary, score, findings list, and remediation steps.
- You can annotate, tag, and integrate the report into your project documentation.

---

## Privacy & Local Storage

- All scan results, options, and histories are stored inside your local ACID-compliant SQLite database.
- Zero analytics, telemetry, or external server logging.

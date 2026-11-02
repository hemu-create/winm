# Legal and Licensing Notes

This is an engineering summary, not legal advice. Consult a lawyer before
shipping anything to users.

## Windows

- WinM must not redistribute Windows images, installation media, keys, or
  Microsoft binaries.
- The installer should download from Microsoft's official channels or build
  media locally from user-supplied sources, verify hashes, and require the user
  to provide a valid license.
- UUP dump is a community tool that fetches Microsoft update packages. Its use
  must be reviewed for compliance with Microsoft's terms; prefer official ISO
  download paths when available.
- Windows updates, drivers, and firmware blobs remain subject to Microsoft's and
  the vendor's terms.
- Do not bypass activation or license enforcement. The TPM/Secure Boot setup
  bypass discussed in this project is about installation compatibility, not
  licensing.

## Apple

- Apple's hardware and firmware are proprietary. WinM should not distribute
  Apple firmware, iBoot, SMC firmware, or device trees extracted from Apple
  software.
- m1n1 interacts with Apple hardware through publicly documented reverse
  engineering and user-supplied firmware. Keep the project's use consistent with
  applicable law and Apple's terms.
- Installing a custom boot object modifies the user's machine. The user is
  responsible for backups, warranty, and recovery.
- The project should not claim Apple endorsement or support.

## m1n1 and Asahi Linux

- m1n1 is GPL-2.0-or-later (with an MIT exception for older versions; see its
  `LICENSE` and `LICENSE.MIT`).
- Asahi Linux components have their own licenses; asahi-installer is MIT.
- WinM's b00t is an independent implementation that uses m1n1's documented boot
  ABI. It does not copy m1n1 code into MIT-licensed files.
- If WinM reuses m1n1 code, that component must be GPL-2.0-or-later and clearly
  isolated. Do not mix GPL and MIT code in a way that misleads users about the
  license.
- Preserve copyright notices and attribution when reusing asahi-installer code
  or documentation.

## EDK2 and UEFI

- EDK2 is BSD-2-Clause-Patent plus per-file licenses.
- UEFI specifications are available under their own terms.
- Microsoft's UEFI CA and Secure Boot key enrollment have policy requirements;
  do not assume entitlement.

## Drivers and signing

- Test signing is a development tool. It reduces platform security and must not
  be presented as a production solution.
- Distributing kernel drivers may require code signing, WHQL/attestation, or
  enterprise certificates.
- HAL extensions are SoC-specific and loaded via ACPI CSRT resource groups; the
  exact API and signing path are not publicly documented. Treat any such
  implementation as high-risk and review it with Microsoft-facing legal/technical
  counsel.
- Do not redistribute proprietary Windows driver binaries.

## Patents and reverse engineering

- Reverse engineering may be restricted by contract, license, or local law.
- Clean-room implementation and publicly available documentation reduce risk but
  do not eliminate it.
- The project should document where each register/interface fact came from and
  avoid incorporating leaked or confidential material.

## Export and cryptography

- The project may include cryptographic code for verification or Secure Boot.
  Review export-control obligations in the relevant jurisdictions.
- Do not bundle keys or certificates that the project is not authorized to
  distribute.

## Warranty and safety

- WinM is experimental software with no warranty.
- A failed install can make a Mac unbootable until recovery/DFU. Users must have
  a backup and a recovery plan.
- Thermal/power bugs can damage hardware. Conservative limits and watchdogs are
  mandatory before stress testing.

## Repository licensing

- Original WinM code and documentation: MIT.
- Any GPL-derived component: GPL-2.0-or-later in a clearly marked directory.
- Third-party code: keep its original license and notices.
- Do not add a Windows or Apple binary to the repository without explicit legal
  review.

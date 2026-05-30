# Security Policy

## Supported versions

meraid is pre-1.0. Security fixes are applied to the latest released `0.x`
version on crates.io.

| Version | Supported |
| ------- | --------- |
| 0.2.x   | ✅        |
| < 0.2   | ❌        |

## Reporting a vulnerability

meraid is a local, offline diagram renderer with a small dependency surface, so
the attack surface is limited. If you do find a security issue (for example a
crash, unbounded memory use, or panic triggered by crafted input):

- Please **do not** open a public issue for anything you consider sensitive.
- Use GitHub's
  [private vulnerability reporting](https://github.com/Binlogo/meraid/security/advisories/new)
  for this repository, or email **binboy886@gmail.com**.

Please include the input that triggers the issue, the version (`meraid
--version`), and your platform. We aim to acknowledge reports within a few days.

Non-sensitive robustness bugs (e.g. malformed input that renders oddly but does
not crash) can be filed as normal issues.

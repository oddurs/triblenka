# Security policy

## Supported versions

The project has not made a release yet. Until 0.1.0, only the tip of `main` is supported.

| Version | Supported |
| ------- | --------- |
| `main`  | yes       |

## Reporting a vulnerability

Report privately through GitHub Security Advisories:

**<https://github.com/oddurs/triblenka/security/advisories/new>**

Please do not open a public issue for a security problem.

Include what you were doing, what happened, and a reproduction if you have one. You should get an
acknowledgement within 7 days and an assessment within 14. If a fix is warranted, the advisory
stays private until it ships, and you will be credited unless you prefer otherwise.

## Scope

Once implementation begins, the areas most worth attention are the ones that handle untrusted
input: content loaders, the markdown pipeline, image processing, server-island prop encryption, and
anything an adapter exposes to the network. Findings in the design itself are welcome too — a
design flaw reported now is far cheaper than one found later.

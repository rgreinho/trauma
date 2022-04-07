# SLSA

[SLSA] is a framework to guide developers and help them secure their software
supply chain.

## SLSA 1

SLSA level 1 does not do much, but helps us get on track to build a resilient
system. We only have 2 requirements to satisfy.

### [Scripted Build]

> All build steps ran using some build service, not on a developer’s
> workstation.

We use GitHub Actions as the automated build system., so we're good here ✅

### [Available Provenance]

> The provenance is available to the consumer in a format that the consumer
> accepts. The format SHOULD be in-toto SLSA Provenance, but another format MAY
> be used if both producer and consumer agree and it meets all the other
> requirements.

We provide provenance using the SLSA [github-actions-demo] and attach it to the
GitHub Release. So we're also good here ✅

### Things to notice

- The attestation will not be uploaded to crates.io, but will live in GitHub
  release. This is not ideal and will complexify the process to verify it.
  Ideally all should be managed by `cargo.`
- The attestation does not mention which commit was used to build the crate.
- Following [cosign]'s convention, we named the attestation
  `trauma.${{ github.sha }}.att`

[available provenance]: https://slsa.dev/spec/v0.1/requirements#available
[cosign]: https://github.com/sigstore/cosign
[github-actions-demo]: https://github.com/slsa-framework/github-actions-demo
[scripted build]: https://slsa.dev/spec/v0.1/requirements#scripted-build
[slsa]: https://slsa.dev/
[slsa levels]: https://slsa.dev/spec/v0.1/levels

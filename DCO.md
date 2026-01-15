# Developer Certificate of Origin

CrabCamera uses the Developer Certificate of Origin (DCO) as its sign-off mechanism for contributions.

## What is the DCO?

The DCO is a lightweight way for contributors to certify that they wrote or otherwise have the right to submit the code they are contributing to the project.

## The DCO Text

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.


Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```

## How to Sign Off

You must sign off on each commit by adding a line like this to your commit message:

```
Signed-off-by: Your Name <your.email@example.com>
```

Git has a built-in flag to do this automatically:

```bash
git commit -s -m "Your commit message"
```

You can also configure Git to always sign off:

```bash
git config --global format.signoff true
```

## Why DCO?

The DCO provides a simple way to track contributions while respecting intellectual property rights. It's less burdensome than a Contributor License Agreement (CLA) while still providing legal clarity.

## Questions?

If you have questions about the DCO, please contact: michaelallenkuykendall@gmail.com

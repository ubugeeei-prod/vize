# @vizejs/doctor

Vize's whole-application health analysis on its own: the `vzdoctor` command.

```bash
vp install -D @vizejs/doctor
```

```bash
vzdoctor
```

`vzdoctor` takes the same arguments as `vize doctor` and reads the same `vite.config.*` and `tsconfig.json`.

This package is one focused entry point into [Vize](https://github.com/ubugeeei-prod/vize). It runs the same release as the `vize` package; install `vize` instead when you use several tools. Guide: https://vizejs.dev/guide/doctor

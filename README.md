BatMUD Client
=============

This is a very early version of a client that can connect to [BatMUD](https://www.bat.org). It will be a very opionated as I'm making it for myself and just for fun. There are many other MUD clients that are more suitable for general use. I even have a separate repository of BatMUD tf triggers [here](https://github.com/oaalto/batmud-tf-trigs) that offer a much better experience for now.

## Manual

The user-facing manual is built with [MkDocs](https://www.mkdocs.org/) (Material theme). See `docs/index.md`, `mkdocs.yml`, and `requirements-docs.txt`.

- **Build then serve** (strict build, then dev server): `./scripts/serve-manual.sh`  
  MkDocs **`serve`** already watches files and **live-reloads** the browser (pass **`--no-livereload`** to turn that off).  
  Optional args are forwarded (e.g. `-a 127.0.0.1:8001`, `--no-livereload`).  
  If `127.0.0.1:8000` is busy, the script picks a free port unless you set `MKDOCS_SERVE_ADDR` or `-a`.

Quick local setup:

```bash
python3 -m venv .venv-docs
.venv-docs/bin/pip install -r requirements-docs.txt
./scripts/serve-manual.sh
```

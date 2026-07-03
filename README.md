# oroot

Browse and enumerate historical paths from old root snapshots.

This is intended to be used alongside setups like [nix-community/impermanence](https://github.com/nix-community/impermanence) with btrfs,
where previous roots are kept under `/btr_pool/old_roots` by default.

## Usage

List the contents of the same path across old roots:

```sh
oroot ls /home/username/.config
```

Enumerate matching directories for integration with other programs:

```sh
oroot enum ~
```

This prints paths like:

```text
/btr_pool/old_roots/2026-06-21_20-16-18/home/username
/btr_pool/old_roots/2026-06-22_09-25-08/home/username
```

`~` is expanded to `$HOME`.

Example with [sxyazi/yazi](https://github.com/sxyazi/yazi),  

```bash
yazi $(oroot enum ~/Downloads | sort -r | head -9)
```

- `sort -r`: sort in reverse order, showing newer roots first.
- `head -n 9`: keep the first 9 entries, since `yazi` only supports 9 tabs.

## Separators

`enum` uses newline separation by default. Use `--separator` to choose another separator:

```sh
oroot enum --separator '\0' ~
oroot enum --separator ' ' ~
oroot enum --separator '\r\n' ~
```

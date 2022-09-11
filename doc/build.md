# BUILD

## Download source code

```
git clone https://gitlab.com/p2kishimoto/bitcoin-alerts.git && cd bitcoin-alerts
```

## Verify commits

Import gpg keys:

```
gpg --keyserver hkps://keys.openpgp.org --recv-keys $(<contrib/verify-commits/trusted-keys)
```

Verify commit:

```
git verify-commit HEAD
```

## Build

Follow instruction for your OS:

* [Unix](build-unix.md) 
# Terminal basics

The terminal runs programs by name and passes them arguments.

## Essential commands

```bash
pwd          # print current directory
ls -lah      # list files, including hidden files
cd path      # change directory
mkdir name   # create directory
cp a b       # copy
mv a b       # move or rename
rm file      # remove file
```

## Paths

- `.` means current directory.
- `..` means parent directory.
- `~` means your home directory.
- `/` separates directories.

## Pipes and redirects

```bash
command > file.txt      # write output to file
command >> file.txt     # append output to file
command | other-command # send output into another command
```

## Permissions

```bash
chmod +x script.sh
./script.sh
```

Use this when a script exists but the shell says it is not executable.

## Safety habits

- Use `pwd` before destructive commands.
- Use `ls` before `rm`.
- Avoid `rm -rf` unless you understand exactly what path it targets.
- Quote paths that contain spaces.

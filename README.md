# Screenshots

![screenshot using nerd icons](asset/nerd.png)
![screenshot using emoji icons](asset/emoji.png)

# Build / Install
```
git clone https://github.com/mudox/tav
cd tav
cargo build
cargo install --path .
```

# Configure `tmux`
Add this to your `~/.tmux.conf` to override the existing function (choose-tree) on `prefix w`:
```
bind-key -T prefix w run-shell tav
```

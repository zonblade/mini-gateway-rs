## Desktop Build

we're using [tauri](https://v2.tauri.app) for desktop build. \
You need to use respective platform you want to build

```
cd router-gui/web-gui
npm i
npm run build
```

then after you build the web, now you need to build the wrapper
```
cd router-client
npm i
npm run tauri build
```

the files wil be resides in `target/release/bundle`
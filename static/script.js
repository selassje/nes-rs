function upload_nes_file_btn_click() {
    document.querySelector("#upload_nes_file").click()
}
function upload_save_file_btn_click() {
    document.querySelector("#upload_save_file").click()
}

function upload_file(file, dir, refreshFn) {
    var reader = new FileReader();
    reader.readAsArrayBuffer(file);
    reader.onload = function (evt) {
        var buf = new Uint8Array(evt.target.result);
        FS.writeFile(dir + "/" + file.name, buf);
        refreshFn();
    }
}

function upload_nes_file() {
    upload_file(this.files[0], "roms", refreshRomFilesList);
}

function upload_save_file() {
    upload_file(this.files[0], "saves", refreshSaveFilesList);
}

function loadSaveState(filename) {
    // Write the filename to a special file that the emulator will check
    const encoder = new TextEncoder();
    FS.writeFile("saves/.load_request", encoder.encode(filename));
}

function saveState(filename) {
    // Write the filename to a special file that the emulator will check to trigger a save
    const encoder = new TextEncoder();
    FS.writeFile("saves/.save_request", encoder.encode(filename));
}

function deleteSaveState(filename) {
    if (confirm(`Delete save file "${filename}"?`)) {
        try {
            FS.unlink(`saves/${filename}`);
            refreshSaveFilesList();
        } catch (e) {
            console.error("Failed to delete file:", e);
        }
    }
}

function loadRom(filename) {
    // Write the filename to a special file that the emulator will check
    const encoder = new TextEncoder();
    FS.writeFile("roms/.load_request", encoder.encode(filename));
}

function deleteRom(filename) {
    if (confirm(`Delete ROM file "${filename}"?`)) {
        try {
            FS.unlink(`roms/${filename}`);
            refreshRomFilesList();
        } catch (e) {
            console.error("Failed to delete file:", e);
        }
    }
}

function alignElements() {
    var bodyWidth = document.querySelector('body').clientWidth;
    var sideWidth = document.querySelector('#sidenav').clientWidth;
    document.querySelector('#main').style.left = sideWidth.toString() + "px";
    document.querySelector('#main').style.width = (bodyWidth - sideWidth).toString() + "px";
}


function getFiles(dir) {
    return FS.readdir(dir)
        .filter(item => item !== '.' && item !== '..' && !item.startsWith('.'))
        .filter(item => FS.isFile(FS.stat(`${dir}/${item}`).mode));
}

function refreshSaveFilesList() {
  const listContainer = document.querySelector("#download_save_files");
  const ul = document.querySelector("#download_list");

  ul.innerHTML = "";
  listContainer.style.display = "none";

  const save_files = getFiles("saves");

  if (save_files.length === 0) {
      alignElements();
      return;
  }

  save_files.forEach(file => {
      const li = document.createElement("li");

      // Play button
      const playBtn = document.createElement("button");
      playBtn.className = "save-action-btn play-btn";
      playBtn.title = "Load this save state";
      playBtn.onclick = () => loadSaveState(file);

      const playIcon = document.createElement("img");
      playIcon.src = "img/play-svgrepo-com.svg";
      playBtn.appendChild(playIcon);

      // Save button
      const saveBtn = document.createElement("button");
      saveBtn.className = "save-action-btn save-btn";
      saveBtn.title = "Overwrite this save state";
      saveBtn.onclick = () => saveState(file);

      const saveIcon = document.createElement("img");
      saveIcon.src = "img/save-svgrepo-com.svg";
      saveBtn.appendChild(saveIcon);

      // Delete button
      const deleteBtn = document.createElement("button");
      deleteBtn.className = "save-action-btn delete-btn";
      deleteBtn.title = "Delete this save state";
      deleteBtn.onclick = () => deleteSaveState(file);

      const deleteIcon = document.createElement("img");
      deleteIcon.src = "img/delete-svgrepo-com.svg";
      deleteBtn.appendChild(deleteIcon);

      // Filename
      const span = document.createElement("span");
      span.textContent = file;

      li.appendChild(playBtn);
      li.appendChild(saveBtn);
      li.appendChild(deleteBtn);
      li.appendChild(span);
      ul.appendChild(li);
  });

  listContainer.style.display = "block";
  alignElements();
}

function refreshRomFilesList() {
  const listContainer = document.querySelector("#download_rom_files");
  const ul = document.querySelector("#rom_list");

  ul.innerHTML = "";
  listContainer.style.display = "none";

  const rom_files = getFiles("roms");

  if (rom_files.length === 0) {
      alignElements();
      return;
  }

  rom_files.forEach(file => {
      const li = document.createElement("li");

      // Play button
      const playBtn = document.createElement("button");
      playBtn.className = "save-action-btn play-btn";
      playBtn.title = "Load this ROM";
      playBtn.onclick = () => loadRom(file);

      const playIcon = document.createElement("img");
      playIcon.src = "img/play-svgrepo-com.svg";
      playBtn.appendChild(playIcon);

      // Delete button
      const deleteBtn = document.createElement("button");
      deleteBtn.className = "save-action-btn delete-btn";
      deleteBtn.title = "Delete this ROM";
      deleteBtn.onclick = () => deleteRom(file);

      const deleteIcon = document.createElement("img");
      deleteIcon.src = "img/delete-svgrepo-com.svg";
      deleteBtn.appendChild(deleteIcon);

      // Filename
      const span = document.createElement("span");
      span.textContent = file;

      li.appendChild(playBtn);
      li.appendChild(deleteBtn);
      li.appendChild(span);
      ul.appendChild(li);
  });

  listContainer.style.display = "block";
  alignElements();
}

FS.rmdir("home/web_user");
FS.rmdir("home");
FS.rmdir("tmp");
FS.mkdir("roms");
FS.mkdir("saves");

document.querySelector("#upload_nes_file").addEventListener("change", upload_nes_file, false);
document.querySelector("#upload_save_file").addEventListener("change", upload_save_file, false);

alignElements();
refreshRomFilesList();
refreshSaveFilesList();

document.addEventListener("contextmenu", (e) => {
  if (e.target.tagName === "CANVAS") {
    e.preventDefault();
  }
});

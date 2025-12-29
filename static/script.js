function upload_nes_file_btn_click() {
    document.querySelector("#upload_nes_file").click()
}
function upload_save_file_btn_click() {
    document.querySelector("#upload_save_file").click()
}

function upload_file(file, dir) {
    var reader = new FileReader();
    reader.readAsArrayBuffer(file);
    reader.onload = function (evt) {
        var buf = new Uint8Array(evt.target.result);
        FS.writeFile(dir + "/" + file.name, buf);
        refreshDownloadList();
    }
}

function upload_nes_file() {
    upload_file(this.files[0], "roms");
}

function upload_save_file() {
    upload_file(this.files[0], "saves");
}

function alignElements() {
    var bodyWidth = document.querySelector('body').clientWidth;
    var sideWidth = document.querySelector('#sidenav').clientWidth;
    document.querySelector('#main').style.left = sideWidth.toString() + "px";
    document.querySelector('#main').style.width = (bodyWidth - sideWidth).toString() + "px";
}


function refreshDownloadList() {
  const listContainer = document.querySelector("#download_save_files");
  const ul = document.querySelector("#download_list");

  ul.innerHTML = "";
  listContainer.style.display = "none";

  const getFiles = (dir) => {
      return FS.readdir(dir)
          .filter(item => FS.isFile(FS.stat(`${dir}/${item}`).mode));
  };

  const save_files = getFiles("saves");

  if (save_files.length === 0) {
      alignElements();
      return;
  }

  save_files.forEach(file => {
      const li = document.createElement("li");
      const link = document.createElement("a");

      const fileContent = FS.readFile(`saves/${file}`);
      link.href = URL.createObjectURL(
          new Blob([fileContent], { type: "application/octet-stream" })
      );

      link.download = file;
      link.textContent = file;

      li.appendChild(link);
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
refreshDownloadList();

document.addEventListener("contextmenu", e => {
  if (e.target.tagName === "CANVAS") {
    e.preventDefault();
  }
});
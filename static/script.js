function upload_nes_file_btn_click() {
    document.getElementById("upload_nes_file").click()
}
function upload_save_file_btn_click() {
    document.getElementById("upload_save_file").click()
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
    upload_file(this.files[0], "games");
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
    var getFiles = (dir) => {
        var files = [];
        var dirContents = FS.readdir(dir);
        var files = dirContents
            .filter((item) => { return FS.isFile(FS.stat(dir + "/" + item).mode) });
        return files;
    }
    var save_files = getFiles("saves");
    var list = document.getElementById("download_save_files");
    list.style.display = "none";
    if (save_files.length > 0) {
        var ul = document.getElementById("download_list");
        ul.innerHTML = "";
        save_files.forEach(element => {
            var entry = document.createElement("li");
            var link = document.createElement("a");
            ul.appendChild(entry);
            link.download = element;
            var fileContent = FS.readFile("saves/" + element);
            var mime = "mime/type" || "application/octet-stream";
            link.href = URL.createObjectURL(new Blob([fileContent], { type: mime }));
            link.innerText = element;
            entry.appendChild(link);
        });
        list.style.display = "block";
    }

    alignElements();
}

FS.rmdir("home/web_user");
FS.rmdir("home");
FS.rmdir("tmp");
FS.mkdir("games");
FS.mkdir("saves");

document.getElementById("upload_nes_file").addEventListener("change", upload_nes_file, false);
document.getElementById("upload_save_file").addEventListener("change", upload_save_file, false);

alignElements();
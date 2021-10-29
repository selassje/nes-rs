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
    }
}

function upload_nes_file() {
    upload_file(this.files[0], "games");
}

function upload_save_file() {
    upload_file(this.files[0], "saves");
}


FS.rmdir("home/web_user");
FS.rmdir("home");
FS.rmdir("tmp");
FS.mkdir("games");
FS.mkdir("saves");

document.getElementById("upload_nes_file").addEventListener("change", upload_nes_file, false);
document.getElementById("upload_save_file").addEventListener("change", upload_save_file, false);
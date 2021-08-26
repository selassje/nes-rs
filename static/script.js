function openNav() {
    document.getElementById("mySidenav").style.width = "50px";
    emulatorCanvas.style.marginLeft = "50px";
}
function closeNav() {
    document.getElementById("mySidenav").style.width = "0";
    emulatorCanvas.style.marginLeft = "0";
}

function upload_nes_file_btn_click() {
    document.getElementById("upload_nes_file").click()
}

function upload_nes_file() {
    let nes_file = this.files[0];
    var reader = new FileReader();
    reader.readAsArrayBuffer(nes_file);
    reader.onload = function (evt) {
        var buf = new Uint8Array(evt.target.result);
        FS.writeFile('games/' + nes_file.name, buf);
    }
}

FS.rmdir("home/web_user");
FS.rmdir("home");
FS.rmdir("tmp");
FS.mkdir("games")
document.getElementById("upload_nes_file").addEventListener("change", upload_nes_file, false);
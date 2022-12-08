function edittoggle(x) {
    div = document.getElementById(x);
    notecontent = div.children[1];
    editarea = div.children[2];
    editb = div.children[3];
    confirmb = div.children[4];
    if(notecontent.classList.contains("block")) {
        //notecontent.classList.replace("block", "hidden");
        //editarea.classList.replace("hidden", "block");
        visswap(notecontent);
        visswap(editarea);
        visswap(editb);
        visswap(confirmb);
    }
}
function visswap(x) {
    if(x.classList.contains("block")) {
        x.classList.replace("block", "hidden");
    } else {
        x.classList.replace("hidden", "block");
    }
}

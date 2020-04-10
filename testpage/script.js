const toBase64 = file => new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.readAsDataURL(file);
    reader.onload = () => resolve({
        filename: file.name,
        data: reader.result.split(',')[1] // getting rid of data:*/*;base64,
    });
    reader.onerror = error => reject(error);
});

function setJsonUploadStatus(type, message) {
    let label = document.getElementById('json-upload-status');
    label.innerHTML = message;
    console.trace(type, message);
    if (type == 'ok')
        label.style.color = '#050';
    else
        label.style.color = '#f00';
}

function uploadJson() {
    const selectedFiles = document.getElementById('json-files').files;
    let fileRequests = [];
    for (let i = 0; i < selectedFiles.length; i++) {
        fileRequests.push(toBase64(selectedFiles[i]))
    }

    Promise.all(fileRequests).then(
        values => new Promise((resolve, reject) => {
            let req = new XMLHttpRequest();
            req.open('POST', '/upload_image', true);
            req.setRequestHeader('Content-type', 'application/json');
            req.onreadystatechange = function() {
                if (this.readyState !== 4) return;
                if (this.status == 201) {
                    resolve(this.responseText);
                } else {
                    reject(this.responseText);
                }
            };
            console.log(values);
            console.log(req);
            req.send(JSON.stringify(values));
        }),
        reason => setJsonUploadStatus('error', JSON.stringify(reason))
    ).then(
        response => setJsonUploadStatus('ok', response),
        reason => setJsonUploadStatus('error', JSON.stringify(reason))
    )
}

document.getElementById('json-send-btn').addEventListener('click', uploadJson);

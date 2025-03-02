const imageManager = {
    dropzone: null,
    fileInput: null, 
    imageGallery: null,
    form: null,

    init(elemento) {
        this.dropzone = elemento.querySelector('.dropzone');
        this.fileInput = this.dropzone.querySelector('input[type="file"]');
        this.imageGallery = elemento.querySelector(".image-gallery.new-images");
        this.form = elemento.querySelector("#imagesForm");

        // Prevenir el comportamiento por defecto del navegador para drag & drop
        ['dragenter', 'dragover', 'dragleave', 'drop'].forEach(eventName => {
            this.dropzone.addEventListener(eventName, this.preventDefaults.bind(this), false);
            document.body.addEventListener(eventName, this.preventDefaults.bind(this), false);
        });

        // Resaltar dropzone cuando se arrastra un archivo sobre ella
        ['dragenter', 'dragover'].forEach(eventName => {
            this.dropzone.addEventListener(eventName, this.highlight.bind(this), false);
        });

        ['dragleave', 'drop'].forEach(eventName => {
            this.dropzone.addEventListener(eventName, this.unhighlight.bind(this), false);
        });

        // Manejar el drop
        this.dropzone.addEventListener('drop', this.handleDrop.bind(this), false);

        // Manejar selección de archivos mediante el input
        this.fileInput.addEventListener('change', this.handleFiles.bind(this), false);
    },

    preventDefaults(e) {
        e.preventDefault();
        e.stopPropagation();
    },

    highlight(e) {
        this.dropzone.classList.add('highlight');
    },

    unhighlight(e) {
        this.dropzone.classList.remove('highlight');
    },

    handleDrop(e) {
        let dt = e.dataTransfer;
        let files = dt.files;
        
        let dataTransfer = new DataTransfer();
        
        if (this.fileInput.files.length > 0) {
            Array.from(this.fileInput.files).forEach(file => dataTransfer.items.add(file));
        }
        
        Array.from(files).forEach(file => dataTransfer.items.add(file));
        
        this.fileInput.files = dataTransfer.files;
        
        this.handleFiles({ target: { files: dataTransfer.files } });
    },

    handleFiles(e) {
        let files = [...e.target.files];
        
        this.imageGallery.innerHTML = '';
        
        let validFiles = files.filter(file => {
            let validTypes = ['image/jpeg', 'image/png', 'image/gif'];
            if (!validTypes.includes(file.type)) {
                alert(`File ${file.name} is not a valid image file`);
                return false;
            }
            return true;
        });

        if (validFiles.length === 0) {
            this.imageGallery.innerHTML = '<p class="no-images">No hay imágenes</p>';
            return;
        }

        validFiles.forEach((file, index) => this.previewFile(file, index));
    },

    previewFile(file, index) {
        let reader = new FileReader();
        reader.readAsDataURL(file);
        
        reader.onloadend = () => {
            let imageContainer = document.createElement('div');
            imageContainer.className = 'image-container';
            imageContainer.dataset.fileIndex = index;
            
            let img = document.createElement('img');
            img.src = reader.result;
            
            let overlay = document.createElement('div');
            overlay.className = 'image-overlay';
            
            let deleteBtn = document.createElement('button');
            deleteBtn.className = 'delete-btn';
            deleteBtn.innerHTML = '<i class="fas fa-trash"></i>';
            deleteBtn.onclick = () => {
                let dataTransfer = new DataTransfer();
                let files = Array.from(this.fileInput.files);
                files.splice(index, 1);
                files.forEach(file => dataTransfer.items.add(file));
                this.fileInput.files = dataTransfer.files;
                
                this.handleFiles({ target: { files: this.fileInput.files } });
            };
            
            overlay.appendChild(deleteBtn);
            imageContainer.appendChild(img);
            imageContainer.appendChild(overlay);
            this.imageGallery.appendChild(imageContainer);
        };
    }
};

// Inicializar cuando el DOM esté listo
/*
document.addEventListener('DOMContentLoaded', () => imageManager.init());

window.addEventListener('popstate', () => imageManager.init());

//evento de htmx para cuado se actualiza el dom
document.addEventListener('htmx:afterSwap', () => imageManager.init());
*/
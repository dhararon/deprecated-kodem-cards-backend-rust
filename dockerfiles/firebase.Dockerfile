FROM node:20-slim

# Instalar Java (necesario para los emuladores de Firebase), curl y unzip para la instalación de Bun
RUN apt-get update && \
    apt-get install -y default-jre-headless curl unzip && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Instalar las herramientas de Firebase
RUN npm install -g firebase-tools@latest

# Instalar Bun (necesario para ejecutar el script de seeding)
RUN curl -fsSL https://bun.sh/install | bash

# Crear directorio de trabajo
WORKDIR /app

# Copiar solo los archivos necesarios
COPY ../../firebase.json .
COPY ../../.firebaserc .
COPY ../../firebase ./firebase
COPY ../../scripts/seed-firebase-users.ts ./scripts/seed-firebase-users.ts

# Exponer los puertos de Firebase
EXPOSE 9099 9199 8080 4000 4400 4500 9150

# Agregar un script para iniciar emuladores y luego ejecutar el script de seeding
COPY ../../scripts/start-emulators-and-seed.sh .
RUN chmod +x start-emulators-and-seed.sh

# Ejecutar el script de inicio personalizado
CMD ["./start-emulators-and-seed.sh"] 
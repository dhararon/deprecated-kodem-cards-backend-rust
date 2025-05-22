/**
 * Script para insertar usuarios de prueba con diferentes roles en el emulador de Firebase
 * 
 * Ejecutar con: bun run scripts/seed-firebase-users.ts
 */

import { initializeApp } from 'firebase-admin/app';
import { getAuth } from 'firebase-admin/auth';
import { getFirestore } from 'firebase-admin/firestore';
import { randomUUID } from 'crypto';
import * as dotenv from 'dotenv';
import { existsSync } from 'fs';
import path from 'path';
import fetch from 'node-fetch';
import { AbortController } from 'abort-controller';

// Cargar variables de entorno desde .env
const envPath = path.resolve(process.cwd(), '.env');
if (existsSync(envPath)) {
    console.log(`Cargando variables de entorno desde: ${envPath}`);
    dotenv.config();
} else {
    console.warn('Archivo .env no encontrado, utilizando variables de entorno del sistema');
}

// Obtener el ID del proyecto de Firebase desde las variables de entorno
const projectId = process.env.VITE_FIREBASE_PROJECT_ID || 'kodemcards';
console.log(`Utilizando Project ID: ${projectId}`);

// Comprobar si estamos en entorno Docker
const isDocker = existsSync('/.dockerenv') || process.env.DOCKER_CONTAINER === 'true';
console.log(`Ejecutando en entorno Docker: ${isDocker ? 'Sí' : 'No'}`);

// Configuración de emuladores
// Si estamos en Docker, usamos los nombres de los servicios como hosts, sino localhost
const authEmulatorHost = isDocker ? 'firebase-emulators' : (process.env.FIREBASE_AUTH_EMULATOR_HOST || 'localhost');
const firestoreEmulatorHost = isDocker ? 'firebase-emulators' : (process.env.FIREBASE_FIRESTORE_EMULATOR_HOST || 'localhost');
const authEmulatorPort = process.env.FIREBASE_AUTH_EMULATOR_PORT || '9099';
const firestoreEmulatorPort = process.env.FIREBASE_FIRESTORE_EMULATOR_PORT || '8080';

// IMPORTANTE: Para el script que se ejecuta FUERA de Docker, siempre usamos localhost
// ya que el script no está dentro de la red de Docker, incluso si los emuladores sí lo están
const authHost = 'localhost';
const firestoreHost = 'localhost';

// URLs para las verificaciones de conexión (siempre usando localhost ya que el script corre fuera de Docker)
const emulatorUIPort = process.env.FIREBASE_EMULATOR_UI_PORT || '4000';
const emulatorUIUrl = `http://localhost:${emulatorUIPort}`;
const authEmulatorUrl = `http://localhost:${authEmulatorPort}`;
const firestoreEmulatorUrl = `http://localhost:${firestoreEmulatorPort}`;

console.log(`Configuración de emuladores para verificación:
- Emulator UI: ${emulatorUIUrl}
- Auth Emulator: ${authEmulatorUrl}
- Firestore Emulator: ${firestoreEmulatorUrl}`);

// Inicializar Firebase Admin con emuladores (para que Firebase Admin SDK se conecte a los emuladores)
process.env.FIRESTORE_EMULATOR_HOST = `${firestoreHost}:${firestoreEmulatorPort}`;
process.env.FIREBASE_AUTH_EMULATOR_HOST = `${authHost}:${authEmulatorPort}`;

// Función para verificar si los emuladores están en ejecución
async function checkEmulatorsRunning() {
    try {
        // Primero verificar que la UI del emulador esté accesible
        console.log(`Verificando UI de emuladores en ${emulatorUIUrl}`);
        try {
            const uiController = new AbortController();
            const uiTimeout = setTimeout(() => uiController.abort(), 3000);
            
            const uiResponse = await fetch(emulatorUIUrl, { 
                method: 'GET',
                signal: uiController.signal
            });
            clearTimeout(uiTimeout);
            
            if (!uiResponse.ok) {
                console.error(`⚠️ UI de emuladores no responde correctamente (status: ${uiResponse.status})`);
                return false;
            }
            console.log('✅ UI de emuladores detectada y funcionando correctamente');
        } catch (error) {
            console.error(`⚠️ No se pudo conectar a la UI de emuladores:`, error);
            return false;
        }
        
        // Si la UI está disponible, asumimos que los demás servicios también lo están
        console.log('✅ Emuladores detectados y funcionando correctamente');
        return true;
    } catch (error) {
        console.error('❌ Error al verificar emuladores:', error);
        console.error('\n⚠️ PROBLEMAS DE CONEXIÓN DETECTADOS:');
        console.error('1. Asegúrate de que Docker está en ejecución con: docker ps');
        console.error('2. Verifica que los contenedores de Firebase están activos');
        console.error('3. Los puertos deben estar accesibles desde tu máquina local:');
        console.error(`   - UI Emuladores: ${emulatorUIUrl}`);
        console.error(`   - Auth: ${authEmulatorUrl}`);
        console.error(`   - Firestore: ${firestoreEmulatorUrl}`);
        console.error('4. Verifica que firebase.json tenga "host": "0.0.0.0" para todos los emuladores');
        console.error('5. Reconstruye los contenedores si es necesario: docker-compose up --build\n');
        return false;
    }
}

// Inicializar la aplicación sin credenciales cuando usamos emuladores
let app, auth, db;

// Definir usuarios de prueba con diferentes roles
interface TestUser {
    email: string;
    password: string;
    displayName: string;
    role: 'admin' | 'user' | 'grader';
    photoURL?: string;
}

const testUsers: TestUser[] = [
    {
        email: 'admin@kodemcards.xyz',
        password: 'admin123',
        displayName: 'Admin User',
        role: 'admin',
        photoURL: 'https://api.dicebear.com/7.x/initials/svg?seed=AU'
    },
    {
        email: 'staff@kodemcards.xyz',
        password: 'staff123',
        displayName: 'Staff User',
        role: 'staff',
        photoURL: 'https://api.dicebear.com/7.x/initials/svg?seed=UR'
    },
    {
        email: 'moderator@kodemcards.xyz',
        password: 'moderator123',
        displayName: 'Moderator User',
        role: 'moderator',
        photoURL: 'https://api.dicebear.com/7.x/initials/svg?seed=UR'
    },
    {
        email: 'user@kodemcards.xyz',
        password: 'user123',
        displayName: 'User',
        role: 'user',
        photoURL: 'https://api.dicebear.com/7.x/initials/svg?seed=UR'
    }
];

// Función para crear un usuario en autenticación y en Firestore
async function createUser(userData: TestUser) {
    try {
        // Primero verificar si el usuario ya existe
        try {
            const userRecord = await auth.getUserByEmail(userData.email);
            console.log(`Usuario ya existe: ${userData.email} (${userRecord.uid})`);
            
            // Actualizar claims para el usuario existente
            await auth.setCustomUserClaims(userRecord.uid, { role: userData.role });
            console.log(`  Claims actualizados para: ${userData.email} (role: ${userData.role})`);
            
            // Actualizar datos en Firestore
            await db.collection('users').doc(userRecord.uid).set({
                email: userData.email,
                displayName: userData.displayName,
                photoURL: userData.photoURL,
                role: userData.role,
                createdAt: new Date().toISOString(),
                updatedAt: new Date().toISOString()
            }, { merge: true });
            
            return userRecord.uid;
        } catch (error) {
            // Si no existe, crearlo
            const userRecord = await auth.createUser({
                email: userData.email,
                password: userData.password,
                displayName: userData.displayName,
                photoURL: userData.photoURL
            });
            
            console.log(`Usuario creado: ${userData.email} (${userRecord.uid})`);
            
            // Agregar custom claims para el rol
            await auth.setCustomUserClaims(userRecord.uid, { role: userData.role });
            console.log(`  Custom claims agregados: role=${userData.role}`);
            
            // Crear documento en Firestore
            await db.collection('users').doc(userRecord.uid).set({
                email: userData.email,
                displayName: userData.displayName,
                photoURL: userData.photoURL,
                role: userData.role,
                createdAt: new Date().toISOString(),
                updatedAt: new Date().toISOString()
            });
            
            return userRecord.uid;
        }
    } catch (error) {
        console.error(`Error al crear/actualizar usuario ${userData.email}:`, error);
        return null;
    }
}

// Función para agregar datos adicionales de usuario
async function createUserData(userId: string, userData: TestUser) {
    try {
        // Crear preferencias por defecto
        const preferences = {
            theme: 'system',
            language: 'es',
            notifications: true
        };
        
        // Crear un deck de ejemplo para cada usuario
        const deckId = randomUUID();
        const deck = {
            id: deckId,
            name: `Deck de ${userData.displayName}`,
            description: 'Este es un deck de ejemplo creado automáticamente',
            cards: [],
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString()
        };
        
        // Crear colección de favoritos vacía
        const favorites: string[] = [];
        
        // Guardar datos completos del usuario
        await db.collection('userData').doc(userId).set({
            id: userId,
            email: userData.email,
            displayName: userData.displayName,
            photoURL: userData.photoURL,
            preferences,
            decks: [deck],
            favorites,
            role: userData.role,
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString()
        });
        
        console.log(`  Datos adicionales creados para: ${userData.email}`);
    } catch (error) {
        console.error(`Error al crear datos adicionales para ${userData.email}:`, error);
    }
}

// Función principal para crear todos los usuarios
async function seedUsers() {
    console.log('🔥 Iniciando inserción de usuarios en emulador de Firebase');
    
    // Verificar que los emuladores estén en ejecución
    const emulatorsRunning = await checkEmulatorsRunning();
    if (!emulatorsRunning) {
        console.error('❌ No se pudo conectar con los emuladores. Abortando.');
        process.exit(1);
    }
    
    try {
        // Inicializar Firebase Admin
        console.log('Inicializando Firebase Admin SDK...');
        app = initializeApp({
            projectId
        }, 'seed-users-app');
        
        auth = getAuth(app);
        db = getFirestore(app);
        
        console.log('Firebase Admin SDK inicializado correctamente.');
        
        // Crear usuarios en secuencia
        for (const user of testUsers) {
            const userId = await createUser(user);
            if (userId) {
                await createUserData(userId, user);
                console.log(`✅ Usuario completo creado: ${user.email} (${user.role})`);
            }
        }
        
        console.log('✅ Todos los usuarios han sido creados exitosamente');
    } catch (error) {
        console.error('❌ Error al crear usuarios:', error);
        console.error('\nPosibles soluciones:');
        console.error('1. Asegúrate de que los emuladores están en ejecución: bun run emulators');
        console.error('2. Verifica que el proyecto ID es correcto: ' + projectId);
        console.error('3. Comprueba que las variables de entorno están bien configuradas');
    }
}

// Ejecutar el script
seedUsers()
    .then(() => {
        console.log('✅ Script finalizado');
        process.exit(0);
    })
    .catch((error) => {
        console.error('❌ Error en la ejecución del script:', error);
        process.exit(1);
    }); 
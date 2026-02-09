# Despliegue CI/CD con GitHub Actions

Esta guia cubre tres estrategias de despliegue automatizado para ApoloTeams.

## Estrategias Disponibles

| | Azure Container Apps | Cloud Run (GCP) | VM / Docker Compose |
|--|---------------------|-----------------|---------------------|
| **Ideal para** | Enterprise, Azure ecosystem | SaaS, multi-tenant | Datacenter cliente, on-premise |
| **Base de datos** | Azure DB for PostgreSQL | Cloud SQL | PostgreSQL en Docker |
| **Escalamiento** | Automatico (0 a N) | Automatico (0 a N) | Manual (vertical) |
| **SSL/HTTPS** | Incluido gratis | Incluido gratis | Nginx + Let's Encrypt |
| **Costo mensual** | ~$20-60 USD (uso bajo) | ~$15-50 USD (uso bajo) | Costo del servidor |
| **WebSockets** | Si (sin timeout fijo) | Si (timeout 1h, reconecta) | Si (sin limites) |
| **Archivos** | Azure Blob Storage / volume | Cloud Storage / GCS FUSE | Disco local (volumen Docker) |
| **Workflow** | `deploy-azure.yml` | `deploy-cloudrun.yml` | `deploy-vm.yml` |

---

## Opcion A: Azure Container Apps (Recomendado)

### Por que Azure Container Apps

- WebSockets sin limite de timeout (ventaja sobre Cloud Run)
- Integracion nativa con Azure Active Directory (ideal para empresas)
- Soporte para volumenes persistentes (Azure Files)
- Red privada con VNet (aislamiento para clientes enterprise)
- Mismo modelo serverless: auto-scaling, pago por uso

### Prerrequisitos

1. Cuenta de Azure con suscripcion activa
2. Azure CLI instalado (`az`)
3. Repositorio en GitHub

### Paso 1: Configurar Azure CLI

```bash
# Instalar Azure CLI
# https://learn.microsoft.com/cli/azure/install-azure-cli

# Login
az login

# Seleccionar suscripcion (si tienes varias)
az account set --subscription "TU_SUBSCRIPTION_ID"

# Registrar los resource providers necesarios (solo la primera vez)
az provider register --namespace Microsoft.ContainerRegistry
az provider register --namespace Microsoft.App
az provider register --namespace Microsoft.OperationalInsights
az provider register --namespace Microsoft.DBforPostgreSQL
az provider register --namespace Microsoft.Storage

# Verificar que estan registrados (esperar hasta que todos digan "Registered")
az provider show --namespace Microsoft.ContainerRegistry --query "registrationState" -o tsv
az provider show --namespace Microsoft.App --query "registrationState" -o tsv
az provider show --namespace Microsoft.OperationalInsights --query "registrationState" -o tsv
az provider show --namespace Microsoft.DBforPostgreSQL --query "registrationState" -o tsv

# Definir variables
export RESOURCE_GROUP=rg-apolo-teams
export LOCATION=eastus2          # o brazilsouth, westus2, etc.
export ACR_NAME=apoloteamsacr    # solo letras minusculas y numeros, unico global
export APP_NAME=apolo-teams
export DB_SERVER=apolo-db-server
export ENV_NAME=apolo-env
```

### Paso 2: Crear Resource Group

```bash
az group create --name $RESOURCE_GROUP --location $LOCATION
```

### Paso 3: Crear Azure Container Registry (ACR)

```bash
# Crear registro (Basic es suficiente para empezar)
az acr create \
  --resource-group $RESOURCE_GROUP \
  --name $ACR_NAME \
  --sku Basic \
  --admin-enabled true

# Obtener credenciales (las necesitaras luego)
az acr credential show --name $ACR_NAME
```

### Paso 4: Crear Storage para PostgreSQL (persistencia)

> **Nota:** Azure PostgreSQL Flexible Server no esta disponible en todas las suscripciones.
> Esta guia usa PostgreSQL como contenedor con Azure Files para persistencia de datos,
> lo cual funciona en cualquier suscripcion y es mas economico.

```bash
# Crear cuenta de almacenamiento
az storage account create \
  --name apolodbstorage \
  --resource-group $RESOURCE_GROUP \
  --location $LOCATION \
  --sku Standard_LRS
# Resultado: provisioningState: "Succeeded", location: "eastus2"

# Crear file share para datos de PostgreSQL
az storage share create \
  --name pgdata \
  --account-name apolodbstorage
# Resultado: { "created": true }

# Obtener la key del storage (se usara en el paso 6)
STORAGE_KEY=$(az storage account keys list \
  --account-name apolodbstorage \
  --resource-group $RESOURCE_GROUP \
  --query "[0].value" -o tsv)
```

### Paso 5: Crear Container Apps Environment

```bash
az containerapp env create \
  --resource-group $RESOURCE_GROUP \
  --name $ENV_NAME \
  --location $LOCATION
# Resultado: provisioningState: "Succeeded"
# defaultDomain: <random>.eastus2.azurecontainerapps.io
```

### Paso 6: Vincular storage y crear PostgreSQL como contenedor

```bash
# Vincular Azure Files al environment
az containerapp env storage set \
  --name $ENV_NAME \
  --resource-group $RESOURCE_GROUP \
  --storage-name pgdata \
  --azure-file-account-name apolodbstorage \
  --azure-file-account-key "$STORAGE_KEY" \
  --azure-file-share-name pgdata \
  --access-mode ReadWrite
# Resultado: name: "pgdata", accessMode: "ReadWrite"

# Crear PostgreSQL como Container App interna (sin acceso externo)
az containerapp create \
  --name apolo-postgres \
  --resource-group $RESOURCE_GROUP \
  --environment $ENV_NAME \
  --image postgres:16-alpine \
  --target-port 5432 \
  --ingress internal \
  --transport tcp \
  --min-replicas 1 \
  --max-replicas 1 \
  --cpu 0.5 \
  --memory 1.0Gi \
  --env-vars \
    POSTGRES_DB=rust_teams \
    POSTGRES_USER=apolo \
    POSTGRES_PASSWORD=TU_PASSWORD_SEGURO_AQUI \
    PGDATA=/var/lib/postgresql/data/pgdata
# Resultado: provisioningState: "Succeeded", runningStatus: "Running"
# FQDN interno: apolo-postgres.internal.<random>.eastus2.azurecontainerapps.io
```

### Paso 7: Construir y desplegar ApoloTeams

> **Nota:** `az acr build` (ACR Tasks) no esta disponible en todos los tipos de suscripcion.
> Si recibes el error `TasksOperationsNotAllowed`, usa el metodo de build local descrito abajo.

```bash
# --- Opcion A: Build en Azure (si tu suscripcion lo permite) ---
# az acr build --registry $ACR_NAME --image apolo-teams:latest .

# --- Opcion B: Build local + push (funciona en cualquier suscripcion) ---
# Login al registro
az acr login --name $ACR_NAME

# Construir imagen localmente (toma ~10-15 min la primera vez por compilacion Rust)
docker build -t ${ACR_NAME}.azurecr.io/apolo-teams:latest .

# Subir imagen al registro
docker push ${ACR_NAME}.azurecr.io/apolo-teams:latest

# Crear la app principal con conexion a PostgreSQL interno
az containerapp create \
  --name $APP_NAME \
  --resource-group $RESOURCE_GROUP \
  --environment $ENV_NAME \
  --image "${ACR_NAME}.azurecr.io/apolo-teams:latest" \
  --registry-server "${ACR_NAME}.azurecr.io" \
  --registry-username $(az acr credential show --name $ACR_NAME --query username -o tsv) \
  --registry-password $(az acr credential show --name $ACR_NAME --query "passwords[0].value" -o tsv) \
  --target-port 8080 \
  --ingress external \
  --transport http \
  --min-replicas 1 \
  --max-replicas 5 \
  --cpu 0.5 \
  --memory 1.0Gi \
  --env-vars \
    SERVER_HOST=0.0.0.0 \
    SERVER_PORT=8080 \
    DATABASE_URL=postgresql://apolo:AiAYwGgZ0rKzJPt9GcrM8PgDhOUtmxJCjbbfCOP7wdmIdKK7qMmaJQQJ99CBACHYHv6Eqg7NAAACAZCRDk9H@apolo-postgres/rust_teams \
    JWT_SECRET=WCUDiO8ppAj7VYuVt1CYKXcscOJbctnj7V7QTLZoi7FLYTA5YfhSrGRCSfPyc11NRfVXB3w14Y4flF2W5VaQCw== \
    RUST_LOG=info,rust_teams_backend=info \
    MAX_FILE_SIZE=104857600 \
    UPLOAD_PATH=/tmp/uploads

# Obtener la URL publica de la aplicacion
az containerapp show \
  --name $APP_NAME \
  --resource-group $RESOURCE_GROUP \
  --query "properties.configuration.ingress.fqdn" -o tsv
# Resultado: apolo-teams.<random>.eastus2.azurecontainerapps.io
```

> **Nota sobre DATABASE_URL:** Dentro del mismo Container Apps Environment,
> los contenedores se comunican por nombre interno. La URL usa `apolo-postgres`
> como hostname (el nombre del container app de PostgreSQL).

### Paso 8: Crear Service Principal para GitHub Actions (CI/CD)

```bash
# Crear service principal con permisos de Contributor
az ad sp create-for-rbac \
  --name "github-apolo-deployer" \
  --role Contributor \
  --scopes /subscriptions/$(az account show --query id -o tsv)/resourceGroups/$RESOURCE_GROUP \
  --json-auth

# IMPORTANTE: Guarda TODO el JSON de salida. Ese es tu AZURE_CREDENTIALS secret.
# Ejemplo de salida:
# {
#   "clientId": "xxx",
#   "clientSecret": "xxx",
#   "subscriptionId": "xxx",
#   "tenantId": "xxx",
#   ...
# }

# Dar permiso de push al ACR
ACR_ID=$(az acr show --name $ACR_NAME --query id -o tsv)
SP_ID=$(az ad sp list --display-name "github-apolo-deployer" --query "[0].id" -o tsv)

az role assignment create \
  --assignee $SP_ID \
  --role AcrPush \
  --scope $ACR_ID
```

### Paso 9: Configurar GitHub Secrets

En tu repositorio GitHub → Settings → Secrets and variables → Actions:

| Secret | Valor | Ejemplo |
|--------|-------|---------|
| `AZURE_CREDENTIALS` | JSON completo del service principal | `{"clientId":"...","clientSecret":"..."}` |
| `AZURE_SUBSCRIPTION_ID` | ID de tu suscripcion | `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` |
| `AZURE_RESOURCE_GROUP` | Nombre del resource group | `rg-apolo-teams` |
| `AZURE_REGION` | Region | `eastus2` |
| `ACR_NAME` | Nombre del Container Registry | `apoloteamsacr` |

### Paso 10: Deploy automatico (CI/CD)

```bash
# Push a main dispara el deploy automaticamente
git push origin main
```

El workflow `deploy-azure.yml` se ejecutara:
1. Build de la imagen Docker
2. Push a Azure Container Registry
3. Deploy a Container Apps
4. URL de produccion en el resumen del workflow

### Configurar dominio personalizado (opcional)

```bash
# Agregar dominio personalizado
az containerapp hostname add \
  --name $APP_NAME \
  --resource-group $RESOURCE_GROUP \
  --hostname teams.tudominio.com

# Vincular certificado SSL (managed)
az containerapp hostname bind \
  --name $APP_NAME \
  --resource-group $RESOURCE_GROUP \
  --hostname teams.tudominio.com \
  --environment $ENV_NAME \
  --validation-method CNAME
```

Luego configura un CNAME en tu DNS apuntando a la FQDN de tu container app.

### Costos estimados (uso bajo, ~20 usuarios)

| Servicio | SKU | Costo aprox./mes |
|----------|-----|------------------|
| Container Apps (app) | 0.5 vCPU, 1GB RAM, 1 replica | ~$15 |
| Container Apps (postgres) | 0.5 vCPU, 1GB RAM, 1 replica | ~$15 |
| Storage Account | Standard LRS | ~$1 |
| Container Registry | Basic | ~$5 |
| **Total** | | **~$36 USD/mes** |

### Arquitectura desplegada

```
Internet
   |
   v  HTTPS (SSL automatico)
┌──────────────────────────────────────────────────────────────┐
│  Azure Container Apps Environment (apolo-env)                │
│                                                              │
│  ┌─────────────────────┐     ┌──────────────────────────┐   │
│  │  apolo-teams         │────▶│  apolo-postgres           │   │
│  │  (ingress: external) │     │  (ingress: internal/tcp)  │   │
│  │  :8080               │     │  :5432                    │   │
│  │  - API REST          │     │  - PostgreSQL 16          │   │
│  │  - SPA React         │     │  - Azure Files (pgdata)   │   │
│  │  - WebSocket         │     └──────────────────────────┘   │
│  └─────────────────────┘                                     │
│                                                              │
│  Azure Container Registry (apoloteamsacr)                    │
│  Azure Storage Account (apolodbstorage)                      │
└──────────────────────────────────────────────────────────────┘
```

### Comandos utiles de administracion

```bash
# Ver logs de la app
az containerapp logs show --name apolo-teams --resource-group rg-apolo-teams --follow

# Ver logs de PostgreSQL
az containerapp logs show --name apolo-postgres --resource-group rg-apolo-teams --follow

# Reiniciar la app (crear nueva revision)
az containerapp revision restart --name apolo-teams --resource-group rg-apolo-teams \
  --revision $(az containerapp revision list --name apolo-teams --resource-group rg-apolo-teams --query "[0].name" -o tsv)

# Escalar manualmente
az containerapp update --name apolo-teams --resource-group rg-apolo-teams \
  --min-replicas 2 --max-replicas 10

# Ver estado de todos los container apps
az containerapp list --resource-group rg-apolo-teams -o table

# Eliminar TODO (para limpiar recursos y evitar costos)
az group delete --name rg-apolo-teams --yes --no-wait
```

---

## Opcion B: Google Cloud Run

### Prerrequisitos

1. Cuenta de Google Cloud con facturacion habilitada
2. Proyecto en GCP
3. Repositorio en GitHub

### Paso 1: Configurar GCP

```bash
# Instalar gcloud CLI (si no lo tienes)
# https://cloud.google.com/sdk/docs/install

# Login y seleccionar proyecto
gcloud auth login
gcloud config set project TU_PROJECT_ID

# Habilitar APIs necesarias
gcloud services enable \
  run.googleapis.com \
  sqladmin.googleapis.com \
  artifactregistry.googleapis.com \
  secretmanager.googleapis.com \
  cloudbuild.googleapis.com

# Definir variables (ajusta a tu region)
export PROJECT_ID=$(gcloud config get-value project)
export REGION=southamerica-east1  # o us-central1, europe-west1, etc.
```

### Paso 2: Crear Artifact Registry (registro de imagenes Docker)

```bash
gcloud artifacts repositories create apolo-teams \
  --repository-format=docker \
  --location=$REGION \
  --description="ApoloTeams Docker images"
```

### Paso 3: Crear Cloud SQL (PostgreSQL)

```bash
# Crear instancia (db-f1-micro es la mas economica)
gcloud sql instances create apolo-db \
  --database-version=POSTGRES_16 \
  --tier=db-f1-micro \
  --region=$REGION \
  --storage-size=10GB \
  --storage-auto-increase

# Crear base de datos
gcloud sql databases create rust_teams --instance=apolo-db

# Crear usuario
gcloud sql users create apolo \
  --instance=apolo-db \
  --password=TU_PASSWORD_SEGURO

# Obtener connection name (lo necesitaras para GitHub Secrets)
gcloud sql instances describe apolo-db --format='value(connectionName)'
# Resultado: project-id:region:apolo-db
```

### Paso 4: Crear Secrets en Secret Manager

```bash
# Guardar la URL de la base de datos
echo -n "postgresql://apolo:TU_PASSWORD@/rust_teams?host=/cloudsql/$PROJECT_ID:$REGION:apolo-db" | \
  gcloud secrets create apolo-db-url --data-file=-

# Guardar JWT secret
echo -n "TU_JWT_SECRET_ALEATORIO_DE_64_CHARS" | \
  gcloud secrets create apolo-jwt-secret --data-file=-
```

### Paso 5: Crear Service Account para GitHub Actions

```bash
# Crear cuenta de servicio
gcloud iam service-accounts create github-deployer \
  --display-name="GitHub Actions Deployer"

SA_EMAIL="github-deployer@${PROJECT_ID}.iam.gserviceaccount.com"

# Asignar permisos necesarios
for ROLE in \
  roles/run.admin \
  roles/artifactregistry.writer \
  roles/iam.serviceAccountUser \
  roles/secretmanager.secretAccessor \
  roles/cloudsql.client; do
  gcloud projects add-iam-policy-binding $PROJECT_ID \
    --member="serviceAccount:${SA_EMAIL}" \
    --role="$ROLE"
done

# Generar clave JSON
gcloud iam service-accounts keys create gcp-key.json \
  --iam-account=$SA_EMAIL

# IMPORTANTE: El contenido de gcp-key.json es tu GCP_SA_KEY secret en GitHub
cat gcp-key.json
```

### Paso 6: Configurar GitHub Secrets

En tu repositorio GitHub → Settings → Secrets and variables → Actions:

| Secret | Valor |
|--------|-------|
| `GCP_PROJECT_ID` | Tu project ID de GCP |
| `GCP_REGION` | `southamerica-east1` (o tu region) |
| `GCP_SA_KEY` | Contenido completo de `gcp-key.json` |
| `CLOUD_SQL_INSTANCE` | `project-id:region:apolo-db` (de paso 3) |

### Paso 7: Desplegar

```bash
# Push a main dispara el deploy automaticamente
git push origin main
```

El workflow `deploy-cloudrun.yml` se ejecutara automaticamente:
1. Build de la imagen Docker
2. Push a Artifact Registry
3. Deploy a Cloud Run
4. URL de produccion disponible en el resumen del workflow

### Configurar dominio personalizado (opcional)

```bash
gcloud run domain-mappings create \
  --service=apolo-teams \
  --domain=teams.tudominio.com \
  --region=$REGION
```

Luego agrega el registro DNS que gcloud te indica.

---

## Opcion C: VM con Docker Compose (On-Premise / Datacenter)

### Prerrequisitos

1. Servidor Linux con Docker y Docker Compose instalados
2. Acceso SSH con clave publica
3. Puerto 80/443 abierto

### Paso 1: Preparar el servidor

```bash
# En el servidor destino
# Instalar Docker
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER

# Crear directorio de la aplicacion
sudo mkdir -p /opt/apolo-teams
sudo chown $USER:$USER /opt/apolo-teams
```

### Paso 2: Configurar GitHub Secrets

En tu repositorio GitHub → Settings → Secrets and variables → Actions:

| Secret | Valor |
|--------|-------|
| `DEPLOY_HOST` | IP o hostname del servidor |
| `DEPLOY_USER` | Usuario SSH (ej: `deploy`) |
| `DEPLOY_SSH_KEY` | Clave privada SSH (contenido de `~/.ssh/id_ed25519`) |
| `DEPLOY_PATH` | `/opt/apolo-teams` |
| `ENV_FILE_CONTENT` | Tu archivo `.env` codificado en base64* |

*Para codificar el `.env`:
```bash
base64 -w 0 .env  # Copiar el resultado como valor del secret
```

### Paso 3: Deploy inicial

1. Copiar `docker-compose.yml` al servidor
2. Copiar `.env` al servidor (basado en `.env.docker`)
3. Ejecutar el workflow "Deploy to VM" manualmente desde GitHub Actions

### Paso 4: Nginx como reverse proxy (HTTPS)

```bash
# En el servidor
sudo apt install -y nginx certbot python3-certbot-nginx

sudo tee /etc/nginx/sites-available/apolo-teams << 'EOF'
server {
    listen 80;
    server_name teams.tudominio.com;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 3600s;
        proxy_send_timeout 3600s;
    }
}
EOF

sudo ln -sf /etc/nginx/sites-available/apolo-teams /etc/nginx/sites-enabled/
sudo nginx -t && sudo systemctl reload nginx

# SSL con Let's Encrypt
sudo certbot --nginx -d teams.tudominio.com
```

---

## Pipelines

### Flujo CI (en cada Pull Request)

```
PR abierto → Frontend (lint + types + build)
           → Backend  (fmt + clippy + test)
           → Docker   (build de validacion)
```

### Flujo CD - Azure Container Apps (push a main)

```
Push a main → Build Docker → Push ACR → Deploy Container Apps
```

### Flujo CD - Cloud Run (push a main)

```
Push a main → Build Docker → Push Artifact Registry → Deploy Cloud Run
```

### Flujo CD - VM (manual)

```
Trigger manual → Build Docker → Push GHCR → SSH deploy → Health check
```

---

## Troubleshooting

### Azure: La app no conecta a PostgreSQL
- Verificar que la firewall rule permite Azure Services:
  ```bash
  az postgres flexible-server firewall-rule list \
    --resource-group rg-apolo-teams --name apolo-db-server
  ```
- Verificar que `sslmode=require` esta en la DATABASE_URL
- Revisar logs: `az containerapp logs show --name apolo-teams --resource-group rg-apolo-teams`

### Azure: Archivos subidos se pierden al escalar
- Container Apps es stateless por defecto
- Opciones:
  - Montar Azure Files como volumen persistente (`az containerapp update --set-env-vars ...`)
  - Migrar uploads a Azure Blob Storage (requiere cambio en backend)

### Cloud Run: WebSocket se desconecta
- Cloud Run tiene timeout maximo de 3600s para conexiones
- El cliente frontend ya implementa reconexion automatica
- Verifica que `--timeout=3600` y `--session-affinity` estan configurados

### Cloud Run: Archivos subidos se pierden
- Cloud Run es stateless, los archivos en disco no persisten
- Opciones:
  - Usar Cloud Storage (requiere cambio en backend)
  - Usar Volume Mount con GCS FUSE (transparente, sin cambios de codigo)

### VM: El health check falla
```bash
# Revisar logs
docker compose logs --tail=100 app

# Verificar que postgres esta listo
docker compose exec postgres pg_isready

# Reiniciar
docker compose restart app
```

### Build falla en CI
```bash
# Verificar localmente
cargo clippy -p backend -p shared -- -D warnings
cd frontend-react && npm run lint && npm run type-check
```

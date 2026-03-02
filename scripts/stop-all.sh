#!/bin/bash

# ============================================================
# Script to stop all Docker containers
# ============================================================
# Usage: ./scripts/stop-all.sh

set -e

echo "🛑 Arrêt de tous les conteneurs Docker..."

# Arrêter tous les conteneurs en cours d'exécution
if [ "$(docker ps -q)" ]; then
    docker stop $(docker ps -q)
    echo "✅ Conteneurs arrêtés"
else
    echo "ℹ️  Aucun conteneur en cours d'exécution"
fi

# Optionnel : supprimer tous les conteneurs arrêtés
read -p "Supprimer tous les conteneurs arrêtés ? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ "$(docker ps -a -q)" ]; then
        docker rm $(docker ps -a -q)
        echo "✅ Conteneurs supprimés"
    else
        echo "ℹ️  Aucun conteneur à supprimer"
    fi
fi

echo "✅ Opération terminée"

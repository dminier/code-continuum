# Documentation des appels statiques et du singleton

## Résumé des modifications

J'ai ajouté la support complet des **appels statiques** et du **pattern singleton** aux exemples Java:

### Fichiers créés/modifiés

1. **ServiceC.java** (créé)
   - Classe avec méthodes statiques utilitaires
   - Méthodes:
     - `logProcessing(serviceName, message)` - Logging statique
     - `encodeData(data)` - Encodage des données
     - `decodeData(encodedData)` - Décodage des données
     - `isValidDataFormat(data)` - Validation du format
     - `generateUniqueId(prefix)` - Génération d'ID unique
     - `transformBatch(items)` - Transformation par lot
     - `computeChecksum(data)` - Calcul de checksum

2. **ServiceA.java** (modifié)
   - Utilise maintenant les méthodes statiques de ServiceC:
     - `processData()` - Appelle `ServiceC.isValidDataFormat()`, `ServiceC.generateUniqueId()`, `ServiceC.logProcessing()`
     - `executeWorkflow()` - Appelle `ServiceC.computeChecksum()`
     - `batchProcess()` - Appelle `ServiceC.transformBatch()`

3. **ServiceB.java** (modifié)
   - Utilise maintenant les méthodes statiques de ServiceC:
     - `transformData()` - Appelle `ServiceC.encodeData()`, `ServiceC.logProcessing()`
     - `validateResult()` - Appelle `ServiceC.isValidDataFormat()`
     - `processComplexData()` - Appelle `ServiceC.generateUniqueId()`

4. **ServiceCSingleton.java** (créé)
   - Pattern singleton qui encapsule ServiceC
   - Méthodes:
     - `getInstance()` - Obtient l'instance unique
     - `trackMethodCall()` - Suit les appels de méthodes
     - `encodeData()` - Wrapper autour de ServiceC.encodeData()
     - `decodeData()` - Wrapper autour de ServiceC.decodeData()
     - `generateUniqueId()` - Wrapper autour de ServiceC.generateUniqueId()
     - `getCallCount()` - Obtient le nombre total d'appels

5. **StaticCallsExample.java** (créé)
   - Exemple démonstratif complet montrant:
     - Les appels statiques directs à ServiceC
     - Comment ServiceA utilise ServiceC
     - Comment ServiceB utilise ServiceC
     - L'utilisation du pattern singleton

## Types d'appels statiques présents

### 1. Appels statiques directs
```java
ServiceC.logProcessing("ServiceA", "Message");
String encoded = ServiceC.encodeData(data);
boolean isValid = ServiceC.isValidDataFormat(input);
int checksum = ServiceC.computeChecksum(processed);
String uniqueId = ServiceC.generateUniqueId("SA");
String[] results = ServiceC.transformBatch(items);
```

### 2. Appels à travers un singleton
```java
ServiceCSingleton singleton = ServiceCSingleton.getInstance();
singleton.encodeData("data");
singleton.generateUniqueId("prefix");
```

## Flux d'appels visibles dans l'extraction

Avec le nouvel extracteur statique Rust, les flux suivants seront visibles:

```
ServiceA.processData() 
  → ServiceC.isValidDataFormat()        (CALL_STATIC)
  → ServiceC.generateUniqueId()         (CALL_STATIC)
  → ServiceC.logProcessing()            (CALL_STATIC)
  → ServiceB.transformData()            (CALL)

ServiceA.executeWorkflow()
  → ServiceC.computeChecksum()          (CALL_STATIC)
  → ServiceC.logProcessing()            (CALL_STATIC)

ServiceA.batchProcess()
  → ServiceC.transformBatch()           (CALL_STATIC)
  → ServiceC.logProcessing()            (CALL_STATIC)

ServiceB.transformData()
  → ServiceC.encodeData()               (CALL_STATIC)
  → ServiceC.logProcessing()            (CALL_STATIC)

ServiceB.validateResult()
  → ServiceC.isValidDataFormat()        (CALL_STATIC)
  → ServiceC.logProcessing()            (CALL_STATIC)

ServiceB.processComplexData()
  → ServiceC.generateUniqueId()         (CALL_STATIC)
  → ServiceC.logProcessing()            (CALL_STATIC)
```

## Exécution de l'exemple

Pour tester les exemples:

```bash
cd /workspaces/code-continuum/examples/backend/java

# Compiler
javac ServiceC.java ServiceA.java ServiceB.java ServiceCSingleton.java StaticCallsExample.java

# Exécuter la démonstration complète
java backend.java.StaticCallsExample

# Ou exécuter les tests individuels
java backend.java.ServiceA
java backend.java.ServiceB
```

## Schéma de la relation

```
┌─────────────┐
│ ServiceA    │
├─────────────┤
│ - name      │
│ - serviceB  │
├─────────────┤
│ processData()──┐
│ execute..()  │  ├─→ ServiceB (dependency)
│ batchProcess()│  │
└─────────────┘  │
       ↓         │
  ServiceC.method_statiques()

┌─────────────┐
│ ServiceB    │
├─────────────┤
│ - name      │
│ - opCount   │
├─────────────┤
│ transformData()──┐
│ validateResult() ├─→ ServiceC (static calls)
│ processComplex() │
└─────────────┘  │
       ↓         │
  ServiceC.method_statiques()

┌─────────────────┐
│ ServiceC        │ (Utility service)
├─────────────────┤
│ + logProcessing()    [STATIC]
│ + encodeData()       [STATIC]
│ + decodeData()       [STATIC]
│ + isValidDataFormat()[STATIC]
│ + generateUniqueId() [STATIC]
│ + transformBatch()   [STATIC]
│ + computeChecksum()  [STATIC]
└─────────────────┘

┌──────────────────────┐
│ ServiceCSingleton    │
├──────────────────────┤
│ - instance [STATIC]  │
│ - callCount          │
├──────────────────────┤
│ + getInstance()      [STATIC]
│ + encodeData()
│ + decodeData()
│ + generateUniqueId()
│ + getCallCount()
└──────────────────────┘
```

## Points clés

- ✅ **Appels statiques directs** : ServiceA et ServiceB appellent des méthodes statiques de ServiceC
- ✅ **Logging centralisé** : Tous les appels utilisent `ServiceC.logProcessing()`
- ✅ **Singleton pattern** : `ServiceCSingleton` encapsule l'accès à ServiceC
- ✅ **Métadonnées** : Chaque appel peut être tracé avec les services d'appels/appelés

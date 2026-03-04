      *> Copybook: constantes partagées entre SalesReport.cbl et CustomerProc.cbl
      *> Inclus via COPY WSCONSTANTS dans les programmes principaux
       01 WS-APP-NAME       PIC X(20) VALUE 'CODE-CONTINUUM-DEMO'.
       01 WS-VERSION        PIC X(5)  VALUE '1.0.0'.
       01 WS-MAX-RECORDS    PIC 9(6)  VALUE 999999.
       01 WS-TRUE           PIC X     VALUE 'Y'.
       01 WS-FALSE          PIC X     VALUE 'N'.

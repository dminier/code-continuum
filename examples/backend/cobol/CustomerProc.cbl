      *> Fixture: programme appelé par SalesReport via CALL 'CUSTPROC'
      *> Permet de tester la résolution inter-programmes des CALL edges
       IDENTIFICATION DIVISION.
       PROGRAM-ID. CUSTPROC.
       AUTHOR. CODE-CONTINUUM.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 WS-STATUS         PIC X(2)  VALUE '00'.
       01 WS-RECORD-COUNT   PIC 9(6)  VALUE ZERO.

       COPY WSCONSTANTS.

       LINKAGE SECTION.
       01 LK-CUSTOMER-REC.
          05 LK-CUST-ID     PIC X(10).
          05 LK-CUST-NAME   PIC X(50).

       PROCEDURE DIVISION USING LK-CUSTOMER-REC.

       MAIN-SECTION SECTION.
           PERFORM VALIDATE-PARAGRAPH.
           PERFORM FETCH-PARAGRAPH.
           GOBACK.

       VALIDATE-PARAGRAPH.
           IF LK-CUST-ID = SPACES
               MOVE 'ER' TO WS-STATUS
           END-IF.

       FETCH-PARAGRAPH.
           ADD 1 TO WS-RECORD-COUNT.
           MOVE 'JOHN DOE' TO LK-CUST-NAME.

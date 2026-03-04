      *> Fixture: programme principal COBOL illustrant les construits extraits
      *> - PROGRAM-ID -> NodeKind::Module
      *> - SECTIONs   -> NodeKind::Function (kind=section)
      *> - PARAGRAPHs -> NodeKind::Function (kind=paragraph)
      *> - COPY       -> NodeKind::Import
      *> - CALL       -> EdgeRelation::Calls
      *> - PERFORM    -> EdgeRelation::Calls
       IDENTIFICATION DIVISION.
       PROGRAM-ID. SALESRPT.
       AUTHOR. CODE-CONTINUUM.

       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SOURCE-COMPUTER. IBM-MAINFRAME.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 WS-COUNTER        PIC 9(4)  VALUE ZERO.
       01 WS-TOTAL-SALES    PIC 9(8)V99 VALUE ZERO.
       01 WS-DB-HANDLE      PIC X(8)  VALUE SPACES.
       01 WS-CUSTOMER-REC.
          05 WS-CUST-ID     PIC X(10).
          05 WS-CUST-NAME   PIC X(50).

       COPY WSCONSTANTS.

       PROCEDURE DIVISION.

       MAIN-SECTION SECTION.
           PERFORM INIT-PARAGRAPH.
           PERFORM PROCESS-SALES-SECTION.
           PERFORM CLOSE-PARAGRAPH.
           STOP RUN.

       INIT-PARAGRAPH.
           MOVE ZERO TO WS-COUNTER.
           MOVE ZERO TO WS-TOTAL-SALES.
           CALL 'DBACCESS' USING WS-DB-HANDLE.

       PROCESS-SALES-SECTION SECTION.
           PERFORM FETCH-CUSTOMER-PARAGRAPH.
           PERFORM ACCUMULATE-PARAGRAPH.

       FETCH-CUSTOMER-PARAGRAPH.
           CALL 'CUSTPROC' USING WS-CUSTOMER-REC.
           ADD 1 TO WS-COUNTER.

       ACCUMULATE-PARAGRAPH.
           ADD WS-TOTAL-SALES TO WS-TOTAL-SALES.

       CLOSE-PARAGRAPH.
           CALL 'DBACLOSE' USING WS-DB-HANDLE.

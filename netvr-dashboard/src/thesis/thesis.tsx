/** @jsxRuntime classic */
import pdf from '@react-pdf/renderer'
const {
  PDFViewer,
  Document: PDFDocument,
  PDFRenderer,
  StyleSheet,
  Page,
  View,
  Text,
} = pdf

function Document() {
  return (
    <PDFDocument>
      <Page size="A4">
        <View style={styles.section}>
          <Text>Section #1</Text>
        </View>
        <View style={styles.section}>
          <Text>Section #2</Text>
        </View>
      </Page>
    </PDFDocument>
  )
}

export function Thesis() {
  return (
    <PDFViewer
      style={{ display: 'flex', width: '100%', border: 0, flexGrow: 1 }}
    >
      <Document />
    </PDFViewer>
  )
}

const styles = StyleSheet.create({
  page: {
    flexDirection: 'row',
    backgroundColor: '#E4E4E4',
  },
  section: {
    margin: 10,
    padding: 10,
    flexGrow: 1,
  },
})

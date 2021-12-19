/** @jsxRuntime classic */
import pdf from '@react-pdf/renderer'
import { PDFContextProvider, usePDFContext } from './base'
import { registerFonts } from './font'
import { TitlePage } from './title-page'
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
  registerFonts()
  const { lang } = usePDFContext()
  return (
    <PDFDocument>
      <TitlePage
        title={
          lang === 'en'
            ? 'Tracking multiple VR users in a shared physical space'
            : 'Sledování více uživatelů VR světa ve sdíleném fyzickém prostoru'
        }
      />
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
      <PDFContextProvider value={{ lang: 'en' }}>
        <Document />
      </PDFContextProvider>
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

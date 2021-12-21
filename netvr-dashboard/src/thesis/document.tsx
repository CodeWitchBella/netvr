import pdf from '@react-pdf/renderer'
import { Page, usePDFContext, TechnikaText } from './base'
import { LMText, registerFonts } from './font'
import { TitlePage } from './title-page'
const { Document: PDFDocument, StyleSheet, View } = pdf

export function Document() {
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
      <Page style={{ alignItems: 'center', justifyContent: 'flex-end' }}>
        <LMText fontFamily="lmroman10-regular">
          Page intentionally left blank
        </LMText>
      </Page>

      <Page>
        <View style={styles.section}>
          <LMText fontFamily="lmroman12-regular" style={{ fontSize: 12 }}>
            Section #1aeaaeaaa
          </LMText>
        </View>
        <View style={styles.section}>
          <TechnikaText>Section #2</TechnikaText>
        </View>
      </Page>
    </PDFDocument>
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

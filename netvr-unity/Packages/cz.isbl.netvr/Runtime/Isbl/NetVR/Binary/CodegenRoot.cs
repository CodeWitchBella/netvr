using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class CodegenRoot: IEquatable<CodegenRoot>, ICloneable {
        public ReadRemoteDevicesOutput field0;
        public JustInstance field1;
        public Nothing field2;
        public InstanceAndSession field3;
        public RemoteSnapshot field4;
        public StartInput field5;
        public Snapshot field6;
        public InitRemoteObjectsInput field7;
        public GrabInput field8;
        public SetPoseInput field9;
        public OnlyString field10;

        public CodegenRoot(ReadRemoteDevicesOutput _field0, JustInstance _field1, Nothing _field2, InstanceAndSession _field3, RemoteSnapshot _field4, StartInput _field5, Snapshot _field6, InitRemoteObjectsInput _field7, GrabInput _field8, SetPoseInput _field9, OnlyString _field10) {
            if (_field0 == null) throw new ArgumentNullException(nameof(_field0));
            field0 = _field0;
            if (_field1 == null) throw new ArgumentNullException(nameof(_field1));
            field1 = _field1;
            if (_field2 == null) throw new ArgumentNullException(nameof(_field2));
            field2 = _field2;
            if (_field3 == null) throw new ArgumentNullException(nameof(_field3));
            field3 = _field3;
            if (_field4 == null) throw new ArgumentNullException(nameof(_field4));
            field4 = _field4;
            if (_field5 == null) throw new ArgumentNullException(nameof(_field5));
            field5 = _field5;
            if (_field6 == null) throw new ArgumentNullException(nameof(_field6));
            field6 = _field6;
            if (_field7 == null) throw new ArgumentNullException(nameof(_field7));
            field7 = _field7;
            if (_field8 == null) throw new ArgumentNullException(nameof(_field8));
            field8 = _field8;
            if (_field9 == null) throw new ArgumentNullException(nameof(_field9));
            field9 = _field9;
            if (_field10 == null) throw new ArgumentNullException(nameof(_field10));
            field10 = _field10;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            field0.Serialize(serializer);
            field1.Serialize(serializer);
            field2.Serialize(serializer);
            field3.Serialize(serializer);
            field4.Serialize(serializer);
            field5.Serialize(serializer);
            field6.Serialize(serializer);
            field7.Serialize(serializer);
            field8.Serialize(serializer);
            field9.Serialize(serializer);
            field10.Serialize(serializer);
            serializer.decrease_container_depth();
        }

        public int BincodeSerialize(byte[] outputBuffer) => BincodeSerialize(new ArraySegment<byte>(outputBuffer));

        public int BincodeSerialize(ArraySegment<byte> outputBuffer) {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer(outputBuffer);
            Serialize(serializer);
            return serializer.get_buffer_offset();
        }

        public byte[] BincodeSerialize()  {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer();
            Serialize(serializer);
            return serializer.get_bytes();
        }

        public static CodegenRoot Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            CodegenRoot obj = new CodegenRoot(
            	ReadRemoteDevicesOutput.Deserialize(deserializer),
            	JustInstance.Deserialize(deserializer),
            	Nothing.Deserialize(deserializer),
            	InstanceAndSession.Deserialize(deserializer),
            	RemoteSnapshot.Deserialize(deserializer),
            	StartInput.Deserialize(deserializer),
            	Snapshot.Deserialize(deserializer),
            	InitRemoteObjectsInput.Deserialize(deserializer),
            	GrabInput.Deserialize(deserializer),
            	SetPoseInput.Deserialize(deserializer),
            	OnlyString.Deserialize(deserializer));
            deserializer.decrease_container_depth();
            return obj;
        }

        public static CodegenRoot BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static CodegenRoot BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            CodegenRoot value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is CodegenRoot other && Equals(other);

        public static bool operator ==(CodegenRoot left, CodegenRoot right) => Equals(left, right);

        public static bool operator !=(CodegenRoot left, CodegenRoot right) => !Equals(left, right);

        public bool Equals(CodegenRoot other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!field0.Equals(other.field0)) return false;
            if (!field1.Equals(other.field1)) return false;
            if (!field2.Equals(other.field2)) return false;
            if (!field3.Equals(other.field3)) return false;
            if (!field4.Equals(other.field4)) return false;
            if (!field5.Equals(other.field5)) return false;
            if (!field6.Equals(other.field6)) return false;
            if (!field7.Equals(other.field7)) return false;
            if (!field8.Equals(other.field8)) return false;
            if (!field9.Equals(other.field9)) return false;
            if (!field10.Equals(other.field10)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + field0.GetHashCode();
                value = 31 * value + field1.GetHashCode();
                value = 31 * value + field2.GetHashCode();
                value = 31 * value + field3.GetHashCode();
                value = 31 * value + field4.GetHashCode();
                value = 31 * value + field5.GetHashCode();
                value = 31 * value + field6.GetHashCode();
                value = 31 * value + field7.GetHashCode();
                value = 31 * value + field8.GetHashCode();
                value = 31 * value + field9.GetHashCode();
                value = 31 * value + field10.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public CodegenRoot Clone() => (CodegenRoot)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary
